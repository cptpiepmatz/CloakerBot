use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, iter};

use chrono::{DateTime, Local};
use const_format::str_replace;
use derive_more::From;
use futures::future::FutureExt;
use futures::stream::{select, StreamExt};
use serde_json::Value;
use songbird::shards::TwilightMap;
use songbird::Songbird;
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::error::ReceiveMessageError;
use twilight_gateway::stream::ShardEventStream;
use twilight_gateway::Shard;
use twilight_http::client::Client;
use twilight_http::request::Request;
use twilight_http::routing::Route;
use twilight_http::Response;
use twilight_model::application::interaction::application_command;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::marker::{ApplicationMarker, ChannelMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::config::GuildConfig;
use crate::data::stats::GuildStats;
use crate::data::{handle_persist_data, PersistentData};
use crate::data::owner_notifications::OwnerNotified;
use crate::interaction::{handle_interaction, HandleInteractionError};
use crate::sync::{KeyLock, OnceLockExpect, VolatileMap};
use crate::takedown::danger_zones::{check_danger_zones, CheckDangerZonesError};
use crate::takedown::random::{handle_random_check, CheckRandomError};
use crate::takedown::PlannedTakedown;
use crate::util::cache::{CacheUpdateEvent, ObservableCache};
use crate::util::log::info;
use crate::util::ref_access_map::RefAccessMap;
use crate::util::trim::trim;

mod assets;
mod data;
mod interaction;
mod owner_notifications;
mod sync;
mod takedown;
mod util;

// TODO: log messages using allowed_mention([]) to hide mentions

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
    client: Client,
    guild_configs: PersistentData<HashMap<Id<GuildMarker>, GuildConfig>>,
    guild_stats: PersistentData<HashMap<Id<GuildMarker>, GuildStats>>,
    /// Lock per guild to allow only one takedown to happen per guild.
    takedown_locks: KeyLock<Id<GuildMarker>>,
    /// Planned takedowns per guild.
    ///
    /// As only one takedown may happen per guild, a single values is fine.
    planned_takedowns: VolatileMap<Id<GuildMarker>, PlannedTakedown>,
    random_takedown_schedule: PersistentData<HashMap<Id<GuildMarker>, DateTime<Local>>>,
    cache: ObservableCache,
    songbird: Songbird,
    owner_notifications: PersistentData<RefAccessMap<Id<GuildMarker>, Id<UserMarker>, OwnerNotified>>
}

static STATE: OnceLockExpect<State> = OnceLockExpect::new();

static READY: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
enum BotEvent {
    Discord(Result<DiscordEvent, ReceiveMessageError>),
    RandomCheck,
    PersistData,
    CacheUpdate(CacheUpdateEvent)
}

#[tokio::main]
async fn main() {
    let mut shard = Shard::new(ShardId::ONE, DISCORD_TOKEN.to_string(), INTENTS);
    let http = Client::new(DISCORD_TOKEN.to_owned());
    let (cache, cache_rx) = ObservableCache::new();
    OnceLockExpect::set(&STATE, State {
        client: http,
        guild_configs: PersistentData::load()
            .await
            .expect("load should work")
            .unwrap_or_default(),
        guild_stats: PersistentData::load()
            .await
            .expect("load should work")
            .unwrap_or_default(),
        takedown_locks: Default::default(),
        planned_takedowns: Default::default(),
        random_takedown_schedule: Default::default(),
        cache,
        songbird: Songbird::twilight(
            Arc::new(TwilightMap::new(HashMap::from([(
                shard.id().number(),
                shard.sender()
            )]))),
            BOT_USER_ID
        ),
        owner_notifications: Default::default()
    });

    takedown::sounds::prep_sounds()
        .await
        .expect("prepping sounds is necessary");

    interaction::register_commands(&STATE.client.interaction(APPLICATION_ID))
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

    let cache_updates = ReceiverStream::new(cache_rx).map(BotEvent::CacheUpdate);

    let events = select(discord_events, random_check_interval);
    let events = select(events, persist_data_interval);
    let mut events = select(events, cache_updates);

    while let Some(event) = events.next().await {
        tokio::spawn(handle_event(event));
    }
}

#[derive(Debug, From)]
enum HandleEventError {
    Fatal,
    Discord(HandleDiscordEventError),
    Persist(io::Error),
    CheckDangerZones(CheckDangerZonesError),
    RandomCheck(CheckRandomError)
}

async fn handle_event(event: BotEvent) {
    async fn run(event: BotEvent) -> Result<(), HandleEventError> {
        match (READY.load(Ordering::SeqCst), event) {
            // Bot is not ready
            (false, BotEvent::Discord(Err(source))) if source.is_fatal() => {
                return Err(HandleEventError::Fatal)
            }
            (false, BotEvent::Discord(Ok(DiscordEvent::Ready(_)))) => {
                READY.store(true, Ordering::SeqCst);
                info!("Bot is ready");
            }
            (false, _) => {}

            // Bot is ready
            (true, event) => match event {
                BotEvent::Discord(event) => match event {
                    Ok(event) => handle_discord_event(event).await?,
                    Err(source) if source.is_fatal() => return Err(HandleEventError::Fatal),
                    Err(_) => ()
                },
                BotEvent::RandomCheck => handle_random_check().await?,
                BotEvent::PersistData => handle_persist_data().await?,
                BotEvent::CacheUpdate(update) => match update {
                    CacheUpdateEvent::VoiceChannelUpdate(update) => {
                        check_danger_zones(update).await?
                    }
                }
            }
        }

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
    STATE.cache.observe(&event).await;
    match event {
        // clear persistent data from this guild
        DiscordEvent::GuildDelete(_) => {
            // TODO: handle cleanup of deleted guild data
        }

        // handle interactions
        DiscordEvent::InteractionCreate(interaction_create) => {
            handle_interaction(&interaction_create).await?
        }

        _ => ()
    }

    Ok(())
}
