use std::iter;
use std::sync::Mutex;
use std::time::Duration;

use futures::stream::{select, StreamExt};
use tokio_stream::wrappers::IntervalStream;
use twilight_gateway::error::ReceiveMessageError;
use twilight_gateway::stream::ShardEventStream;
use twilight_gateway::Shard;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::marker::{ApplicationMarker, UserMarker};
use twilight_model::id::Id;

use crate::sync::OnceLockExpect;

mod sync;
mod util;

#[cfg(debug_assertions)]
const DISCORD_TOKEN: &str = include_str!("../dev.token");
#[cfg(not(debug_assertions))]
const DISCORD_TOKEN: &str = include_str!("../.token");

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
struct State {}

static STATE: OnceLockExpect<State> = OnceLockExpect::new();

#[derive(Debug)]
enum Event {
    Discord(Result<DiscordEvent, ReceiveMessageError>),
    RandomCheck,
    PersistData
}

#[tokio::main]
async fn main() {
    let mut shard = Shard::new(ShardId::ONE, DISCORD_TOKEN.to_string(), INTENTS);
    let discord_events =
        ShardEventStream::new(iter::once(&mut shard)).map(|(_, event)| Event::Discord(event));

    let random_check_interval =
        IntervalStream::new(tokio::time::interval(RANDOM_CHECK_INTERVAL_DURATION))
            .map(|_| Event::RandomCheck);
    let persist_data_interval =
        IntervalStream::new(tokio::time::interval(PERSIST_DATA_INTERVAL_DURATION))
            .map(|_| Event::PersistData);

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

async fn handle_event(event: Event) {
    async fn run(event: Event) -> Result<(), HandleEventError> {
        match event {
            Event::Discord(event) => match event {
                Ok(event) => handle_discord_event(event).await?,
                Err(source) if source.is_fatal() => return Err(HandleEventError::Fatal),
                Err(_) => ()
            },
            Event::RandomCheck => todo!(),
            Event::PersistData => todo!()
        };

        Ok(())
    }

    run(event).await.unwrap();
}

#[derive(Debug)]
struct HandleDiscordEventError;

impl From<HandleDiscordEventError> for HandleEventError {
    fn from(value: HandleDiscordEventError) -> Self {
        HandleEventError::Discord(value)
    }
}

async fn handle_discord_event(event: DiscordEvent) -> Result<(), HandleDiscordEventError> {
    todo!()
}
