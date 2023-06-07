use std::iter;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use const_format::str_replace;
use futures::stream::{select, StreamExt};
use tokio_stream::wrappers::IntervalStream;
use twilight_gateway::error::ReceiveMessageError;
use twilight_gateway::stream::ShardEventStream;
use twilight_gateway::Shard;
use twilight_http::client::Client as HttpClient;
use twilight_model::application::interaction::application_command;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::marker::{ApplicationMarker, UserMarker};
use twilight_model::id::Id;

use crate::interaction::{handle_interaction, HandleInteractionError};
use crate::sync::OnceLockExpect;

mod assets;
mod interaction;
mod sync;
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
    http: Arc<HttpClient>
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
    let http = Arc::new(HttpClient::new(DISCORD_TOKEN.to_owned()));
    OnceLockExpect::set(&STATE, State { http });

    interaction::register_commands(&STATE.http.interaction(APPLICATION_ID))
        .await
        .expect("registering commands is necessary");

    return;

    let mut shard = Shard::new(ShardId::ONE, DISCORD_TOKEN.to_string(), INTENTS);
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

#[derive(Debug)]
enum HandleEventError {
    Fatal,
    Discord(HandleDiscordEventError)
}

async fn handle_event(event: BotEvent) {
    async fn run(event: BotEvent) -> Result<(), HandleEventError> {
        match event {
            BotEvent::Discord(event) => match event {
                Ok(event) => handle_discord_event(event).await?,
                Err(source) if source.is_fatal() => return Err(HandleEventError::Fatal),
                Err(_) => ()
            },
            BotEvent::RandomCheck => todo!(),
            BotEvent::PersistData => todo!()
        };

        Ok(())
    }

    run(event).await.unwrap();
}

#[derive(Debug)]
enum HandleDiscordEventError {
    Interaction(HandleInteractionError)
}

impl_from_variant!(HandleDiscordEventError, HandleEventError, Discord);

async fn handle_discord_event(event: DiscordEvent) -> Result<(), HandleDiscordEventError> {
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
        DiscordEvent::VoiceStateUpdate(_) => todo!(),

        _ => ()
    }

    Ok(())
}
