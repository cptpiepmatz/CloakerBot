#![allow(unused)]
#![allow(unstable_name_collisions)]

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use std::{io, iter};

use derive_more::From;
use futures::stream::{self, StreamExt, select};
use interaction::COMMANDS_REGISTER;
use miette::IntoDiagnostic;
use songbird::Songbird;
use songbird::shards::TwilightMap;
use sync::Eject;
use takedown::sounds::Sound;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver as TokioMpscReceiver, Sender as TokioMpscSender};
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use tracing::Instrument;
use twilight_gateway::error::{ReceiveMessageError, ReceiveMessageErrorType};
use twilight_gateway::{EventTypeFlags, Shard, StreamExt as TwilightStreamExt};
use twilight_http::client::Client;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::id::Id;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, UserMarker};
use twilight_model::oauth::scope;

use crate::data::Database;
use crate::diagnostics::panic::ExpectReport;
use crate::diagnostics::{logging, panic};
use crate::discord::emojis;
use crate::interaction::{HandleInteractionError, handle_interaction};
use crate::sync::{KeyLock, VolatileMap};
use crate::takedown::PlannedTakedown;
use crate::takedown::danger_zones::{CheckDangerZonesError, check_danger_zones};
use crate::takedown::random::{CheckRandomError, handle_random_check};
use crate::util::cache::{CacheUpdateEvent, ObservableCache};
use crate::util::trim::trim;

mod assets;
mod data;
mod diagnostics;
mod discord;
mod http_server;
mod interaction;
mod link;
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
const INVITE_SCOPES: &[&str] = &[scope::BOT, scope::APPLICATIONS_COMMANDS];

static TOKEN: LazyString = lazy_env!("DISCORD_TOKEN");
const INTENTS: Intents = Intents::GUILDS
    .union(Intents::GUILD_VOICE_STATES)
    .union(Intents::GUILD_MESSAGES)
    .union(Intents::GUILD_INTEGRATIONS);

// "cloak" on a T9 keypad, since "cloaker" exceeds the port range
const HTTP_PORT: u16 = macros::env!("HTTP_PORT", 25625u16);
const HTTP_HOST: &str = macros::env!("HTTP_HOST", "0.0.0.0");
static CLIENT_SECRET: LazyString = lazy_env!("DISCORD_CLIENT_SECRET");
static COOKIE_SECRET: LazyString = lazy_env!("COOKIE_SECRET");
const REDIRECT_URI: &str = macros::env!(
    "DISCORD_REDIRECT_URI",
    dev = "https://dev.cloakerbot.cptpiepmatz.de/callback",
    prod = "https://cloakerbot.cptpiepmatz.de/callback"
);

const RANDOM_CHECK_EVERY: Duration = Duration::from_secs(15);
const BACKUP_INTERVAL: Duration =
    Duration::from_secs(macros::env!("BACKUP_INTERVAL_SECONDS", 300u64));

#[derive(Debug)]
struct State {
    shard: Eject<Shard>,
    client: Client,
    http_client: reqwest::Client,
    database: Database,
    /// Lock per guild to allow only one takedown to happen per guild.
    takedown_locks: KeyLock<Id<GuildMarker>>,
    /// Planned takedowns per guild.
    ///
    /// As only one takedown may happen per guild, a single values is fine.
    planned_takedowns: VolatileMap<Id<GuildMarker>, PlannedTakedown>,
    cache: ObservableCache,
    cache_rx: Eject<TokioMpscReceiver<CacheUpdateEvent>>,
    songbird: Songbird,
    owner_notifications: Mutex<HashMap<Id<UserMarker>, data::owner_notifications::OwnerNotified>>,
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

    let songbird = Songbird::twilight(
        Arc::new(TwilightMap::new(HashMap::from([(shard.id().number(), shard.sender())]))),
        BOT_USER_ID
    );

    let shard = Eject::new(shard);

    let takedown_locks = Default::default();
    let planned_takedowns = Default::default();

    let (cache, cache_rx) = ObservableCache::new();
    let cache_rx = Eject::new(cache_rx);

    let database = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(Database::init("data/cloaker.sqlite"))
            .expect_report("could not init database")
    });

    State {
        shard,
        client,
        http_client,
        database,
        takedown_locks,
        planned_takedowns,
        cache,
        cache_rx,
        songbird,
        owner_notifications: Default::default(),
        linked_role_locks: Default::default()
    }
});

#[tokio::main]
async fn main() -> miette::Result<()> {
    panic::init();
    logging::init().into_diagnostic()?;
    tracing::info!("starting cloakerbot");

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| miette::miette!("could not select the TLS crypto provider"))?;

    let state = LazyLock::force(&STATE);

    // prepare backups
    if !(Duration::from_secs(1)..=Duration::from_secs(86400)).contains(&BACKUP_INTERVAL) {
        return Err(miette::miette!("BACKUP_INTERVAL_SECONDS must be between 1 and 86400"));
    }
    let backups = tokio::spawn(state.database.maintain_backups(BACKUP_INTERVAL).instrument(
        tracing::info_span!("database_backups", interval_seconds = BACKUP_INTERVAL.as_secs())
    ));

    let shard = state.shard.eject().expect_report("shard went missing");
    let cache_rx = state.cache_rx.eject().expect_report("cache_rx went missing");

    // ensure that technically lazy elements are evaluated
    tracing::debug!("loading bundled sounds");
    Sound::load_eagerly();
    tracing::info!("bundled sounds loaded");
    tracing::debug!("registering application commands");
    LazyLock::force(&COMMANDS_REGISTER);
    tracing::info!("application commands registered");
    LazyLock::force(&emojis::EMOJIS_REGISTER);
    tracing::debug!("registering linked-role metadata");
    link::application_role_connection::register_metadata()
        .await
        .map_err(|error| miette::miette!("registering linked-role metadata failed: {error:?}"))?;
    tracing::info!("linked-role metadata registered");

    // spawn server for account linking
    let http_server =
        tokio::spawn(http_server::serve().instrument(tracing::info_span!("http_server")));
    let linked_roles = tokio::spawn(
        link::application_role_connection::maintain_connections()
            .instrument(tracing::info_span!("linked_role_maintenance"))
    );

    let discord_events = stream::unfold(shard, |mut shard| async {
        shard
            .next_event(EventTypeFlags::all())
            .await
            .map(|event| (BotEvent::Discord(event.map(Box::new)), shard))
    });

    let random_check_interval = IntervalStream::new(tokio::time::interval(RANDOM_CHECK_EVERY))
        .map(|_| BotEvent::RandomCheck);

    let cache_updates = ReceiverStream::new(cache_rx).map(BotEvent::CacheUpdate);

    let events = select(discord_events, random_check_interval);
    let mut events = select(events, cache_updates);

    let event_handler = tokio::spawn(async {
        futures::pin_mut!(events);

        loop {
            match events.next().await {
                None => return, // event stream ended
                Some(BotEvent::Discord(Ok(event))) if matches!(*event, DiscordEvent::Ready(_)) => {
                    break;
                }
                Some(event @ BotEvent::Discord(Err(_))) => handle_event(event).await,
                _ => continue
            }
        }

        tracing::info!("bot is ready");
        while let Some(event) = events.next().await {
            tokio::spawn(handle_event(event).in_current_span());
        }
    });

    tokio::select! {
        server = http_server => server.into_diagnostic()?.into_diagnostic()?,
        handler = event_handler => handler.into_diagnostic()?,
        roles = linked_roles => roles.into_diagnostic()?,
        backup = backups => backup.into_diagnostic()?
    }

    Err(miette::miette!("a bot worker stopped unexpectedly"))
}

#[derive(Debug)]
enum BotEvent {
    Discord(Result<Box<DiscordEvent>, ReceiveMessageError>),
    RandomCheck,
    CacheUpdate(CacheUpdateEvent)
}

#[derive(Debug, Error, From)]
enum HandleEventError {
    #[error("gateway requires reconnection")]
    Fatal,

    #[error("Discord event failed: {0:?}")]
    Discord(HandleDiscordEventError),

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

#[tracing::instrument(skip_all, fields(event_kind = match &event {
    BotEvent::Discord(Ok(event)) => event.kind().name().unwrap_or("unknown"),
    BotEvent::Discord(Err(_)) => "gateway_error",
    BotEvent::RandomCheck => "random_check",
    BotEvent::CacheUpdate(_) => "cache_update"
}))]
async fn handle_event(event: BotEvent) {
    async fn run(event: BotEvent) -> Result<(), HandleEventError> {
        match event {
            BotEvent::Discord(event) => match event {
                Ok(event) => handle_discord_event(*event).await?,
                Err(source) => HandleEventError::check_fatal(source)?
            },
            BotEvent::RandomCheck => handle_random_check().await?,
            BotEvent::CacheUpdate(update) => match update {
                CacheUpdateEvent::VoiceChannelUpdate(update) => check_danger_zones(update).await?
            }
        }

        Ok(())
    }

    if let Err(error) = run(event).await &&
        !matches!(error, HandleEventError::Discord(HandleDiscordEventError::Interaction(_)))
    {
        tracing::error!(?error, "event handling failed");
    }
}

#[derive(Debug, From)]
enum HandleDiscordEventError {
    Database(sqlx::Error),
    Interaction(HandleInteractionError),
    OwnerNotification(owner_notifications::OwnerNotificationError)
}

async fn handle_discord_event(event: DiscordEvent) -> Result<(), HandleDiscordEventError> {
    // observe voice changes before songbird waits on an active call
    STATE.cache.observe(&event).await;
    STATE.songbird.process(&event).await;
    match event {
        // clear persistent data from this guild
        DiscordEvent::GuildDelete(deleted) if deleted.unavailable != Some(true) => {
            let _lock = STATE.takedown_locks.lock(deleted.id).await;
            STATE.database.remove_guild(deleted.id).await?;
            tracing::info!(guild_id = %deleted.id, "removed persistent data for departed guild");
            STATE.owner_notifications.lock().await.clear();
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
        LazyLock::new(|| {
            use crate::diagnostics::panic::ExpectReport;
            std::env::var($env).expect_report(concat!("could not read env variable ", $env))
        })
    };
}

use lazy_env;
