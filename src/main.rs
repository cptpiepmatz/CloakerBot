use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, iter};

use const_format::str_replace;
use derive_more::From;
use futures::stream::{select, StreamExt};
use serde_json::Value;
use songbird::shards::TwilightMap;
use songbird::Songbird;
use tokio_stream::wrappers::IntervalStream;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::error::ReceiveMessageError;
use twilight_gateway::stream::ShardEventStream;
use twilight_gateway::Shard;
use twilight_http::client::Client as HttpClient;
use twilight_http::request::Request;
use twilight_http::routing::Route;
use twilight_http::Response;
use twilight_model::application::interaction::application_command;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::config::GuildConfig;
use crate::data::{handle_persist_data, PersistentData};
use crate::interaction::{handle_interaction, HandleInteractionError};
use crate::sync::{KeyLock, OnceLockExpect, VolatileMap};
use crate::takedown::PlannedTakedown;
use crate::util::log::info;
use crate::util::trim::trim;

mod assets;
mod data;
mod interaction;
mod sync;
mod takedown;
mod util;

#[cfg(debug_assertions)]
const DISCORD_TOKEN: &str = trim!(include_str!("../dev.token"));
#[cfg(not(debug_assertions))]
const DISCORD_TOKEN: &str = trim!(include_str!("../.token"));

#[cfg(debug_assertions)]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(604066644465287198);
#[cfg(not(debug_assertions))]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(772259037580361729);

const BOT_USER_ID: Id<UserMarker> = APPLICATION_ID.cast();

const INTENTS: Intents = Intents::GUILDS
    .union(Intents::GUILD_VOICE_STATES)
    .union(Intents::GUILD_MESSAGES)
    .union(Intents::GUILD_INTEGRATIONS);

const RANDOM_CHECK_INTERVAL_DURATION: Duration = Duration::from_secs(15);

const PERSIST_DATA_INTERVAL_DURATION: Duration = Duration::from_millis(500);

#[derive(Debug)]
struct State {
    http: HttpClient,
    guild_configs: PersistentData<HashMap<Id<GuildMarker>, GuildConfig>>,
    /// Lock per guild to allow only one takedown to happen per guild.
    takedown_locks: KeyLock<Id<GuildMarker>>,
    /// Planned takedowns per guild.
    ///
    /// As only one takedown may happen per guild, a single values is fine.
    planned_takedowns: VolatileMap<Id<GuildMarker>, PlannedTakedown>,
    cache: InMemoryCache,
    songbird: Songbird
}

static STATE: OnceLockExpect<State> = OnceLockExpect::new();

#[derive(Debug)]
enum BotEvent {
    Discord(Result<DiscordEvent, ReceiveMessageError>),
    RandomCheck,
    PersistData
}

#[tokio::main]
async fn main() {
    let mut shard = Shard::new(ShardId::ONE, DISCORD_TOKEN.to_string(), INTENTS);
    let http = HttpClient::new(DISCORD_TOKEN.to_owned());
    OnceLockExpect::set(&STATE, State {
        http,
        guild_configs: PersistentData::load()
            .await
            .expect("load should work")
            .unwrap_or_default(),
        takedown_locks: Default::default(),
        planned_takedowns: Default::default(),
        cache: InMemoryCache::builder()
            .resource_types(
                ResourceType::CHANNEL &
                    ResourceType::GUILD &
                    ResourceType::MEMBER &
                    ResourceType::USER &
                    ResourceType::VOICE_STATE
            )
            .build(),
        songbird: Songbird::twilight(
            Arc::new(TwilightMap::new(HashMap::from([(
                shard.id().number(),
                shard.sender()
            )]))),
            BOT_USER_ID
        )
    });

    takedown::sounds::prep_sounds()
        .await
        .expect("prepping sounds is necessary");

    interaction::register_commands(&STATE.http.interaction(APPLICATION_ID))
        .await
        .expect("registering commands is necessary");

    let discord_events =
        ShardEventStream::new(iter::once(&mut shard)).map(|(_, event)| BotEvent::Discord(event));

    let random_check_interval =
        IntervalStream::new(tokio::time::interval(RANDOM_CHECK_INTERVAL_DURATION))
            .map(|_| BotEvent::RandomCheck);
    let persist_data_interval =
        IntervalStream::new(tokio::time::interval(PERSIST_DATA_INTERVAL_DURATION))
            .map(|_| BotEvent::PersistData);

    let events = select(discord_events, random_check_interval);
    let mut events = select(events, persist_data_interval);

    while let Some(event) = events.next().await {
        tokio::spawn(handle_event(event));
    }
}

#[derive(Debug, From)]
enum HandleEventError {
    Fatal,
    Discord(HandleDiscordEventError),
    Persist(io::Error)
}

async fn handle_event(event: BotEvent) {
    async fn run(event: BotEvent) -> Result<(), HandleEventError> {
        match event {
            BotEvent::Discord(event) => match event {
                Ok(event) => handle_discord_event(event).await?,
                Err(source) if source.is_fatal() => return Err(HandleEventError::Fatal),
                Err(_) => ()
            },
            BotEvent::RandomCheck => (), // TODO: handle random checks
            BotEvent::PersistData => handle_persist_data().await?
        };

        Ok(())
    }

    run(event).await.unwrap();
}

#[derive(Debug, From)]
enum HandleDiscordEventError {
    Interaction(HandleInteractionError)
}

async fn handle_discord_event(event: DiscordEvent) -> Result<(), HandleDiscordEventError> {
    STATE.songbird.process(&event).await;
    STATE.cache.update(&event);
    match event {
        // log when ready
        DiscordEvent::Ready(_) => info!("Bot is ready"),

        // clear persistent data from this guild
        DiscordEvent::GuildDelete(_) => todo!(),

        // handle interactions
        DiscordEvent::InteractionCreate(interaction_create) => {
            handle_interaction(&interaction_create).await?
        }

        // handle user joins a voice channel
        DiscordEvent::VoiceStateUpdate(_) => {
            // TODO
        }

        _ => ()
    }

    Ok(())
}
