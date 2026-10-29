#![allow(unused)]

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use std::{io, iter};

use derive_more::From;
use futures::stream::{self, StreamExt, select};
use interaction::COMMANDS_REGISTER;
use miette::{Diagnostic, IntoDiagnostic};
use songbird::Songbird;
use songbird::shards::TwilightMap;
use sync::Eject;
use takedown::sounds::Sound;
use thiserror::Error;
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
use crate::data::stats::PersistentAppStats;
use crate::data::{PersistentData, handle_persist_data};
use crate::interaction::{HandleInteractionError, handle_interaction};
use crate::sync::{KeyLock, VolatileMap};
use crate::takedown::PlannedTakedown;
use crate::takedown::danger_zones::{CheckDangerZonesError, check_danger_zones};
use crate::takedown::random::{CheckRandomError, handle_random_check};
use crate::util::cache::{CacheUpdateEvent, ObservableCache};
use crate::util::trim::trim;

mod assets;
mod data;
mod interaction;
mod link;
mod logging;
mod owner_notifications;
mod ranks;
mod sync;
mod takedown;
mod util;

#[cfg(all(feature = "dev", feature = "prod"))]
compile_error!("the feature `dev` and `prod` are mutually exclusive");

#[cfg(all(not(feature = "dev"), not(feature = "prod")))]
compile_error!("one of the features `dev` or `prod` are required");

#[rustfmt::skip]
const CLIENT_ID: u64 = macros::env!(
    "DISCORD_CLIENT_ID", 
    dev = 1369284129031458906u64, 
    prod = 772259037580361729u64
);

const APPLICATION_ID: Id<ApplicationMarker> = Id::new(CLIENT_ID);
const BOT_USER_ID: Id<UserMarker> = Id::new(CLIENT_ID);

static TOKEN: LazyString = lazy_env!("DISCORD_TOKEN");
const INTENTS: Intents = Intents::GUILDS
    .union(Intents::GUILD_VOICE_STATES)
    .union(Intents::GUILD_MESSAGES)
    .union(Intents::GUILD_INTEGRATIONS);

const OAUTH_PORT: u16 = macros::env!("DISCORD_OAUTH_PORT", 9000u16);
static CLIENT_SECRET: LazyString = lazy_env!("DISCORD_CLIENT_SECRET");
static COOKIE_SECRET: LazyString = lazy_env!("COOKIE_SECRET");
const REDIRECT_URI: &str = macros::env!(
    "DISCORD_REDIRECT_URI",
    dev = "https://dev.cloakerbot.cptpiepmatz.de/callback",
    prod = "https://cloakerbot.cptpiepmatz.de/callback"
);

const RANDOM_CHECK_EVERY: Duration = Duration::from_secs(15);
const PERSIST_DATA_EVERY: Duration = Duration::from_millis(500);

#[derive(Debug)]
struct State {
    shard: Eject<Shard>,
    client: Client,
    http_client: reqwest::Client,
    ready: AtomicBool,
    guild_configs: PersistentData<PersistentGuildConfig>,
    stats: PersistentData<PersistentAppStats>,
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
    owner_notifications: PersistentData<PersistentOwnerNotifications>,
    owner_notification_locks: KeyLock<Id<UserMarker>>,
    linked_roles: PersistentData<data::linked_roles::PersistentLinkedRoles>,
    linked_role_locks: KeyLock<Id<UserMarker>>
}

static STATE: LazyLock<State> = LazyLock::new(|| {
    let discord_token = TOKEN.deref();
    let mut shard = Shard::new(ShardId::ONE, discord_token.to_owned(), INTENTS);
    let client = Client::builder()
        .token(discord_token.to_owned())
        .default_allowed_mentions(twilight_model::channel::message::AllowedMentions::default())
        .build();

    let http_client = reqwest::Client::new();

    let ready = AtomicBool::new(false);

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
    let owner_notifications = PersistentData::load_blocking().unwrap().unwrap_or_default();

    let (cache, cache_rx) = ObservableCache::new();
    let cache_rx = Eject::new(cache_rx);

    State {
        shard,
        client,
        http_client,
        ready,
        guild_configs,
        stats: guild_stats,
        takedown_locks,
        planned_takedowns,
        random_takedown_schedule,
        cache,
        cache_rx,
        songbird,
        owner_notifications,
        owner_notification_locks: Default::default(),
        linked_roles: PersistentData::load_blocking().unwrap().unwrap_or_default(),
        linked_role_locks: Default::default()
    }
});

#[tokio::main]
async fn main() -> miette::Result<()> {
    // console_subscriber::init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| miette::miette!("could not select the TLS crypto provider"))?;

    let state = LazyLock::force(&STATE);
    let shard = state.shard.eject().expect("shard went missing");
    let cache_rx = state.cache_rx.eject().expect("cache_rx went missing");

    // ensure that technically lazy elements are evaluated
    Sound::load_eagerly();
    LazyLock::force(&COMMANDS_REGISTER);
    link::application_role_connection::register_metadata()
        .await
        .map_err(|error| miette::miette!("registering linked-role metadata failed: {error:?}"))?;

    let http_server = tokio::spawn(link::serve());
    let linked_roles = tokio::spawn(link::application_role_connection::maintain_connections());

    let discord_events = stream::unfold(shard, |mut shard| async {
        shard.next_event(EventTypeFlags::all()).await.map(|event| (BotEvent::Discord(event), shard))
    });

    let random_check_interval = IntervalStream::new(tokio::time::interval(RANDOM_CHECK_EVERY))
        .map(|_| BotEvent::RandomCheck);
    let persist_data_interval = IntervalStream::new(tokio::time::interval(PERSIST_DATA_EVERY))
        .map(|_| BotEvent::PersistData);

    let cache_updates = ReceiverStream::new(cache_rx).map(BotEvent::CacheUpdate);

    let events = select(discord_events, random_check_interval);
    let events = select(events, persist_data_interval);
    let mut events = select(events, cache_updates);

    let event_handler = tokio::spawn(async {
        futures::pin_mut!(events);
        while let Some(event) = events.next().await {
            tokio::spawn(handle_event(event));
        }
    });

    tokio::select! {
        server = http_server => server.into_diagnostic()?.into_diagnostic()?,
        handler = event_handler => handler.into_diagnostic()?,
        roles = linked_roles => roles.into_diagnostic()?
    }
    Err(miette::miette!("a bot worker stopped unexpectedly"))
}

#[derive(Debug)]
enum BotEvent {
    Discord(Result<DiscordEvent, ReceiveMessageError>),
    RandomCheck,
    PersistData,
    CacheUpdate(CacheUpdateEvent)
}

#[derive(Debug, Error, Diagnostic, From)]
enum HandleEventError {
    #[error("gateway requires reconnection")]
    #[diagnostic(help("check the Discord connection and restart the bot if it cannot reconnect"))]
    Fatal,
    #[error("Discord event failed: {0:?}")]
    Discord(HandleDiscordEventError),
    #[error("persisting bot data failed")]
    #[diagnostic(help("check that the data directory is writable and storage is available"))]
    Persist(#[source] io::Error),
    #[error("danger-zone check failed: {0:?}")]
    CheckDangerZones(CheckDangerZonesError),
    #[error("random-takedown check failed: {0:?}")]
    RandomCheck(CheckRandomError)
}

impl HandleEventError {
    fn check_fatal(source: ReceiveMessageError) -> Result<(), Self> {
        type E = ReceiveMessageErrorType;
        match source.kind() {
            E::Reconnect => Err(HandleEventError::Fatal),
            _ => Ok(())
        }
    }
}

async fn handle_event(event: BotEvent) {
    async fn run(event: BotEvent) -> Result<(), HandleEventError> {
        match (STATE.ready.load(Ordering::Acquire), event) {
            // Bot is not ready
            (false, BotEvent::Discord(Err(source))) => HandleEventError::check_fatal(source)?,
            (false, BotEvent::Discord(Ok(DiscordEvent::Ready(_)))) => {
                STATE.ready.store(true, Ordering::Release);
                macros::info!("Bot is ready");
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

    if let Err(error) = run(event).await {
        let report = miette::Report::new(error);
        macros::error!("{report:?}");
    }
}

#[derive(Debug, From)]
enum HandleDiscordEventError {
    Interaction(HandleInteractionError),
    OwnerNotification(crate::util::ClientError)
}

async fn handle_discord_event(event: DiscordEvent) -> Result<(), HandleDiscordEventError> {
    // observe voice changes before songbird waits on an active call
    STATE.cache.observe(&event).await;
    STATE.songbird.process(&event).await;
    match event {
        // clear persistent data from this guild
        DiscordEvent::GuildDelete(deleted) if deleted.unavailable != Some(true) => {
            let _lock = STATE.takedown_locks.lock(deleted.id).await;
            STATE.guild_configs.write().await.remove(&deleted.id);
            STATE.stats.write().await.takedowns.remove(&deleted.id);
            STATE.random_takedown_schedule.write().await.remove(&deleted.id);
            STATE.owner_notifications.write().await.remove_ref(&deleted.id);
        }

        // handle interactions
        DiscordEvent::InteractionCreate(interaction_create) => {
            handle_interaction(&interaction_create).await?
        }

        DiscordEvent::MessageCreate(message) => owner_notifications::on_message(&message).await?,

        _ => ()
    }

    Ok(())
}

type LazyString = LazyLock<String>;
macro_rules! lazy_env {
    ($env:literal) => {
        LazyLock::new(|| std::env::var($env).expect(concat!("missing env variable ", $env)))
    };
}

use lazy_env;
