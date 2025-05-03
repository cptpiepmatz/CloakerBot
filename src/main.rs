#![allow(unused)]

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use std::{env, io, iter};

use derive_more::From;
use futures::stream::{self, StreamExt, select};
use interaction::COMMANDS_REGISTER;
use songbird::Songbird;
use songbird::shards::TwilightMap;
use sync::Eject;
use takedown::sounds::Sound;
use tokio::sync::mpsc::{Receiver as TokioMpscReceiver, Sender as TokioMpscSender};
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use twilight_gateway::error::{ReceiveMessageError, ReceiveMessageErrorType};
use twilight_gateway::{EventTypeFlags, Shard, StreamExt as TwilightStreamExt};
use twilight_http::client::Client;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, UserMarker};

use crate::data::config::PersistentGuildConfig;
use crate::data::owner_notifications::PersistentOwnerNotifications;
use crate::data::random::PersistentTakedownSchedule;
use crate::data::stats::PersistentGuildStats;
use crate::data::{PersistentData, handle_persist_data};
use crate::interaction::{HandleInteractionError, handle_interaction};
use crate::sync::{KeyLock, VolatileMap};
use crate::takedown::PlannedTakedown;
use crate::takedown::danger_zones::{CheckDangerZonesError, check_danger_zones};
use crate::takedown::random::{CheckRandomError, handle_random_check};
use crate::util::cache::{CacheUpdateEvent, ObservableCache};
use crate::util::log::info;
use crate::util::trim::trim;

mod assets;
mod data;
mod interaction;
mod owner_notifications;
mod ranks;
mod sync;
mod takedown;
mod util;

// TODO: log messages using allowed_mention([]) to hide mentions

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
    shard: Eject<Shard>,
    client: Client,
    guild_configs: PersistentData<PersistentGuildConfig>,
    guild_stats: PersistentData<PersistentGuildStats>,
    /// Lock per guild to allow only one takedown to happen per guild.
    takedown_locks: KeyLock<Id<GuildMarker>>,
    /// Planned takedowns per guild.
    ///
    /// As only one takedown may happen per guild, a single values is fine.
    planned_takedowns: VolatileMap<Id<GuildMarker>, PlannedTakedown>,
    random_takedown_schedule: PersistentData<PersistentTakedownSchedule>,
    cache: ObservableCache,
    cache_rx: Eject<TokioMpscReceiver<CacheUpdateEvent>>,
    songbird: Songbird,
    owner_notifications: PersistentData<PersistentOwnerNotifications>
}

static READY: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
enum BotEvent {
    Discord(Result<DiscordEvent, ReceiveMessageError>),
    RandomCheck,
    PersistData,
    CacheUpdate(CacheUpdateEvent)
}

static STATE: LazyLock<State> = LazyLock::new(|| {
    let discord_token = env::var("DISCORD_TOKEN").expect("missing env variable DISCORD_TOKEN");
    let mut shard = Shard::new(ShardId::ONE, discord_token.clone(), INTENTS);
    let client = Client::new(discord_token);

    let songbird = Songbird::twilight(
        Arc::new(TwilightMap::new(HashMap::from([(shard.id().number(), shard.sender())]))),
        BOT_USER_ID
    );

    let shard = Eject::new(shard);

    let guild_configs = PersistentData::load_blocking().unwrap().unwrap_or_default();
    let guild_stats = PersistentData::load_blocking().unwrap().unwrap_or_default();
    let random_takedown_schedule = PersistentData::load_blocking().unwrap().unwrap_or_default();

    let takedown_locks = Default::default();
    let planned_takedowns = Default::default();
    let owner_notifications = Default::default();

    let (cache, cache_rx) = ObservableCache::new();
    let cache_rx = Eject::new(cache_rx);

    State {
        shard,
        client,
        guild_configs,
        guild_stats,
        takedown_locks,
        planned_takedowns,
        random_takedown_schedule,
        cache,
        cache_rx,
        songbird,
        owner_notifications
    }
});

#[tokio::main]
async fn main() {
    let state = STATE.deref();
    let shard = state.shard.eject().expect("shard went missing");
    let cache_rx = state.cache_rx.eject().expect("cache_rx went missing");

    // ensure that technically lazy elements are evaluated
    Sound::load_eagerly();
    COMMANDS_REGISTER.deref();

    return;

    let discord_events = stream::unfold(shard, |mut shard| async {
        shard.next_event(EventTypeFlags::all()).await.map(|event| (BotEvent::Discord(event), shard))
    });

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

    futures::pin_mut!(events);
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

impl HandleEventError {
    fn check_fatal(source: ReceiveMessageError) -> Result<(), Self> {
        type E = ReceiveMessageErrorType;
        match source.kind() {
            E::Reconnect | E::WebSocket => Err(HandleEventError::Fatal),
            _ => Ok(())
        }
    }
}

async fn handle_event(event: BotEvent) {
    async fn run(event: BotEvent) -> Result<(), HandleEventError> {
        match (READY.load(Ordering::SeqCst), event) {
            // Bot is not ready
            (false, BotEvent::Discord(Err(source))) => HandleEventError::check_fatal(source)?,
            (false, BotEvent::Discord(Ok(DiscordEvent::Ready(_)))) => {
                READY.store(true, Ordering::SeqCst);
                info!("Bot is ready");
            }
            (false, _) => {}

            // Bot is ready
            (true, event) => match event {
                BotEvent::Discord(event) => match event {
                    Ok(event) => handle_discord_event(event).await?,
                    Err(source) => HandleEventError::check_fatal(source)?
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
