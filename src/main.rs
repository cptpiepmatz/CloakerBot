use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use futures::stream::StreamExt;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::Cluster;
use twilight_http::client::InteractionClient;
use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::gateway::Intents;
use twilight_model::id::marker::{ApplicationMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::user::User;

use crate::interactions::{commands, handle_interaction};
use crate::preparations::file_extract::extract_files;
use crate::preparations::java::find_java_executable;

mod assets;
mod embedded_files;
mod interactions;
mod preparations;

#[cfg(debug_assertions)]
const DISCORD_TOKEN: &str = include_str!("../dev.token");
#[cfg(not(debug_assertions))]
const DISCORD_TOKEN: &str = include_str!("../.token");

#[cfg(debug_assertions)]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(604066644465287198);
#[cfg(not(debug_assertions))]
const APPLICATION_ID: Id<ApplicationMarker> = Id::new(772259037580361729);

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

pub struct TakedownPlan {
    initiator_id: Option<Id<UserMarker>>,
    target_id: Id<UserMarker>,
    countered: bool
}

pub struct Com {
    http: HttpClient,
    event_sender: Sender<Event>,
    shared_data: RwLock<SharedData>
}

pub struct SharedData {
    planned_takedowns: BTreeMap<Id<GuildMarker>, VecDeque<TakedownPlan>>
}

#[tokio::main]
async fn main() {
    // let java_exec = find_java_executable().unwrap();
    // extract_files().unwrap();

    // create http client and register commands
    let http_client = HttpClient::new(String::from(DISCORD_TOKEN.trim()));
    commands::register_commands(&http_client.interaction(APPLICATION_ID))
        .await
        .expect("set commands");

    let intents = Intents::GUILDS |
        Intents::GUILD_VOICE_STATES |
        Intents::GUILD_MESSAGES |
        Intents::GUILD_INTEGRATIONS;

    let (cluster, mut discord_events) = Cluster::new(String::from(DISCORD_TOKEN.trim()), intents)
        .await
        .unwrap();
    let cluster = Arc::new(cluster);

    let cluster_spawn = Arc::clone(&cluster);
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    let shared_data = RwLock::new(SharedData {
        planned_takedowns: BTreeMap::new()
    });

    let (event_sender, mut events) = tokio::sync::mpsc::channel::<Event>(100);

    let com = Arc::new(Com {
        http: HttpClient::new(String::from(DISCORD_TOKEN.trim())),
        event_sender,
        shared_data
    });

    let mut events = futures::stream::select(
        discord_events.map(|item| Event::Discord(item.1)),
        ReceiverStream::new(events)
    );
    while let Some(event) = events.next().await {
        if let Event::Discord(event) = &event {
            cache.update(event);
        }

        tokio::spawn(handle_event(event, Arc::clone(&com)));
    }
}

async fn handle_event(event: Event, com: Arc<Com>) {
    match event {
        Event::Discord(DiscordEvent::ShardConnected(_)) => {
            println!("Connected");
        }
        Event::Discord(DiscordEvent::InteractionCreate(interaction_create)) => {
            let interaction = (*interaction_create).0;
            handle_interaction(interaction, com).await;
        }
        // Other events here...
        _ => {}
    }
}
