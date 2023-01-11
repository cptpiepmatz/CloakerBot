use std::collections::{BTreeMap, HashMap, VecDeque};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Command;
use std::sync::Arc;

use dashmap::DashMap;
use futures::stream::StreamExt;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::opt::Strict;
use surrealdb::Surreal;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use twilight_cache_inmemory::{InMemoryCache as EventCache, ResourceType};
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_lavalink::Lavalink;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::Channel;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::Intents;
use twilight_model::guild::{Member, Role};
use twilight_model::id::marker::{
    ApplicationMarker, ChannelMarker, GuildMarker, RoleMarker, UserMarker
};
use twilight_model::id::Id;

use crate::interactions::{commands, handle_interaction};
use crate::preparations::file_extract::extract_files;
use crate::preparations::java::find_java_executable;

mod assets;
mod db;
mod embedded_files;
mod interactions;
mod permissions;
mod preparations;

#[cfg(debug_assertions)]
const DISCORD_TOKEN: &str = include_str!("../dev.token");
#[cfg(not(debug_assertions))]
const DISCORD_TOKEN: &str = include_str!("../.token");

#[cfg(debug_assertions)]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(604066644465287198);
#[cfg(not(debug_assertions))]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(772259037580361729);

const BOT_USER_ID: Id<UserMarker> = APPLICATION_ID.cast();

const LAVALINK_AUTH: &str = "youshallnotpass";

const INTENTS: Intents = Intents::GUILDS
    .union(Intents::GUILD_VOICE_STATES)
    .union(Intents::GUILD_MESSAGES)
    .union(Intents::GUILD_INTEGRATIONS);

const EVENT_CACHE_RESOURCE_TYPES: ResourceType = ResourceType::VOICE_STATE
    .union(ResourceType::CHANNEL)
    .union(ResourceType::MEMBER)
    .union(ResourceType::ROLE);

pub enum Event {
    /// events from the discord gateway
    Discord(DiscordEvent),

    /// preparation of a takedown, may be initiated by either a user or the
    /// random attack
    TakedownPrep,

    /// check for the takedown strike
    TakedownStrike { interaction: Interaction },

    /// check if random takedowns should be queued
    RandomCheck
}

#[derive(Debug, Default)]
pub struct HttpCache {
    guild_members: DashMap<(Id<GuildMarker>, Id<UserMarker>), Member>,
    roles: DashMap<Id<GuildMarker>, HashMap<Id<RoleMarker>, Role>>,
    channels: DashMap<Id<GuildMarker>, HashMap<Id<ChannelMarker>, Channel>>
}

pub struct TakedownPlan {
    initiator_id: Option<Id<UserMarker>>,
    target_id: Id<UserMarker>,
    countered: bool
}

pub struct Com {
    db: Surreal<Db>,
    http: HttpClient,
    lavalink: Lavalink,
    event_sender: Sender<Event>,
    event_cache: EventCache,
    http_cache: HttpCache,
    planned_takedowns: DashMap<Id<GuildMarker>, VecDeque<TakedownPlan>>
}

#[tokio::main]
async fn main() {
    let java_exec = find_java_executable().unwrap();
    extract_files().unwrap();
    let _lavalink_process = Command::new(java_exec)
        .arg("-jar")
        .arg("Lavalink.jar")
        .spawn()
        .expect("spawn lavalink process");
    let lavalink = Lavalink::new(BOT_USER_ID, 1);
    lavalink
        .add(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2333),
            LAVALINK_AUTH
        )
        .await
        .unwrap();

    // create http client and register commands
    let http_client = HttpClient::new(String::from(DISCORD_TOKEN.trim()));
    commands::register_commands(&http_client.interaction(APPLICATION_ID))
        .await
        .expect("set commands");

    let (cluster, discord_events) = Cluster::new(String::from(DISCORD_TOKEN.trim()), INTENTS)
        .await
        .unwrap();
    let cluster = Arc::new(cluster);

    let cluster_spawn = Arc::clone(&cluster);
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let cache = EventCache::builder()
        .resource_types(EVENT_CACHE_RESOURCE_TYPES)
        .build();

    let (event_sender, events) = tokio::sync::mpsc::channel::<Event>(100);

    let db = Surreal::new::<Mem>(()).await.expect("create db");
    db.use_ns("cptpiepmatz")
        .use_db("cloaker_bot")
        .await
        .expect("set ns and db");

    let com = Arc::new(Com {
        db,
        http: HttpClient::new(String::from(DISCORD_TOKEN.trim())),
        lavalink,
        event_sender,
        event_cache: cache,
        http_cache: HttpCache::default(),
        planned_takedowns: DashMap::new()
    });

    let mut events = futures::stream::select(
        discord_events.map(|item| Event::Discord(item.1)),
        ReceiverStream::new(events)
    );

    while let Some(event) = events.next().await {
        // TODO: add some error handling here
        tokio::spawn(handle_event(event, Arc::clone(&com)));
    }
}

async fn handle_event(event: Event, com: Arc<Com>) -> anyhow::Result<()> {
    Ok(match event {
        Event::Discord(event) => {
            com.event_cache.update(&event);
            com.lavalink.process(&event).await.unwrap();
            match event {
                DiscordEvent::ShardConnected(_) => {
                    println!("Connected");
                }
                DiscordEvent::InteractionCreate(interaction_create) => {
                    let interaction = (*interaction_create).0;
                    handle_interaction(interaction, com).await?;
                }
                _ => {}
            }
        }
        Event::TakedownPrep => {
            todo!()
        }
        _ => {}
    })
}
