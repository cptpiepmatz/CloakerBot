use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use futures::stream::StreamExt;
use lazy_static::lazy_static;
use songbird::input::Input;
use songbird::Songbird;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::{IntervalStream, ReceiverStream};
use twilight_cache_inmemory::{InMemoryCache as EventCache, ResourceType};
use twilight_gateway::{Cluster, Shard};
use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::Intents;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, InteractionMarker, UserMarker};
use twilight_model::id::Id;

use crate::cache::http::Cache as HttpCache;
use crate::cell::InitCell;
use crate::db::DataStore;
use crate::interaction::{application_command, handle_interaction};
use crate::sounds::Sounds;
use crate::sync::TaggedMutex;
use crate::takedown::random::handle_random_check;
use crate::takedown::{
    handle_takedown_init, handle_takedown_strike, PlannedTakedowns, TakedownType
};

mod assets;
mod cache;
mod cell;
mod db;
mod interaction;
mod log;
mod permissions;
mod sounds;
mod sync;
mod takedown;

#[cfg(debug_assertions)]
const DISCORD_TOKEN: &str = include_str!("../dev.token");
#[cfg(not(debug_assertions))]
const DISCORD_TOKEN: &str = include_str!("../.token");

#[cfg(debug_assertions)]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(604066644465287198);
#[cfg(not(debug_assertions))]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(772259037580361729);

const BOT_USER_ID: Id<UserMarker> = APPLICATION_ID.cast();

lazy_static! {
    // TODO: make this const when https://github.com/rust-lang/rust/issues/82485 lands
    static ref LAVALINK_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2333);

    static ref GUILD_VOICE_MUTEX: TaggedMutex<Id<GuildMarker>> = TaggedMutex::new();
}

const INTENTS: Intents = Intents::GUILDS
    .union(Intents::GUILD_VOICE_STATES)
    .union(Intents::GUILD_MESSAGES)
    .union(Intents::GUILD_INTEGRATIONS);

const EVENT_CACHE_RESOURCE_TYPES: ResourceType = ResourceType::VOICE_STATE
    .union(ResourceType::CHANNEL)
    .union(ResourceType::MEMBER)
    .union(ResourceType::ROLE)
    .union(ResourceType::USER)
    .union(ResourceType::GUILD);

static STATE: InitCell<State> = InitCell::new();

#[derive(Debug)]
pub enum Event {
    /// events from the discord gateway
    Discord(DiscordEvent),

    /// preparation of a takedown, may be initiated by either a user, the
    /// random attack or a danger zone
    TakedownInit {
        target_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
        kind: TakedownType
    },

    /// check for the takedown strike
    TakedownStrike { guild_id: Id<GuildMarker> },

    /// check if random takedowns should be queued
    RandomCheck
}

pub struct State {
    ds: DataStore,
    http: HttpClient,
    cluster: Arc<Cluster>,
    songbird: Songbird,
    sounds: Sounds,
    event_sender: Sender<Event>,
    event_cache: EventCache,
    http_cache: HttpCache,
    planned_takedowns: PlannedTakedowns,
    tracked_interactions: DashMap<Id<InteractionMarker>, Interaction>
}

#[tokio::main]
async fn main() {
    // create http client and register commands
    let http_client = HttpClient::new(String::from(DISCORD_TOKEN.trim()));
    application_command::register_commands(&http_client.interaction(APPLICATION_ID))
        .await
        .expect("set commands");

    let (cluster, discord_events) = Cluster::new(DISCORD_TOKEN.trim().to_owned(), INTENTS)
        .await
        .expect("could not create cluster");
    let cluster = Arc::new(cluster);
    cluster.up().await;

    let songbird = Songbird::twilight(cluster.clone(), BOT_USER_ID);
    let sounds = Sounds::prepare().await.expect("preparing sounds failed");

    let cache = EventCache::builder()
        .resource_types(EVENT_CACHE_RESOURCE_TYPES)
        .build();

    let (event_sender, events) = tokio::sync::mpsc::channel::<Event>(100);

    let ds = DataStore::open().await.expect("create db");
    let db = Surreal::new::<Mem>(()).await.expect("create db");
    db.use_ns("cptpiepmatz")
        .use_db("cloaker_bot")
        .await
        .expect("set ns and db");

    InitCell::init(&STATE, State {
        ds,
        http: HttpClient::new(String::from(DISCORD_TOKEN.trim())),
        cluster,
        songbird,
        sounds,
        event_sender,
        event_cache: cache,
        http_cache: HttpCache::default(),
        planned_takedowns: DashMap::default(),
        tracked_interactions: DashMap::default()
    });

    let mut events = futures::stream::select(
        futures::stream::select(
            discord_events.map(|(_, event)| Event::Discord(event)),
            ReceiverStream::new(events)
        ),
        IntervalStream::new(tokio::time::interval(Duration::from_secs(15)))
            .map(|_| Event::RandomCheck)
    );

    while let Some(event) = events.next().await {
        // TODO: add some error handling here
        tokio::spawn(handle_event(event));
    }
}

async fn handle_event(event: Event) {
    async fn run(event: Event) -> anyhow::Result<()> {
        match event {
            Event::Discord(event) => handle_discord_event(event).await?,
            Event::RandomCheck => handle_random_check().await?,
            Event::TakedownInit {
                target_id,
                guild_id,
                kind
            } => {
                handle_takedown_init(target_id, guild_id, kind).await?;
            }
            Event::TakedownStrike { guild_id } => handle_takedown_strike(guild_id).await?
        };

        Ok(())
    }

    run(event).await.unwrap();
}

pub async fn handle_discord_event(event: DiscordEvent) -> anyhow::Result<()> {
    STATE.event_cache.update(&event);
    //STATE.lavalink.process(&event).await?;
    match event {
        DiscordEvent::ShardConnected(_) => {
            println!("Connected");
        }
        DiscordEvent::InteractionCreate(interaction_create) => {
            let interaction = (*interaction_create).0;
            handle_interaction(interaction).await?;
        }
        _ => {}
    }

    Ok(())
}

pub async fn user_is_bot(user_id: Id<UserMarker>) -> anyhow::Result<bool> {
    if let Some(user) = STATE.event_cache.user(user_id) {
        return Ok(user.bot);
    }
    let http_cache = &STATE.http_cache;
    let user = http_cache.fetch_user(user_id).await?;
    Ok(user.bot)
}
