use std::error::Error;
use std::sync::Arc;

use futures::stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::gateway::event::Event as DiscordEvent;
use twilight_model::gateway::Intents;

use crate::commands::set_commands;
use crate::preparations::file_extract::extract_files;
use crate::preparations::java::find_java_executable;

mod assets;
mod commands;
mod embedded_files;
mod preparations;

#[cfg(debug_assertions)]
const DISCORD_TOKEN: &str = include_str!("../dev.token");
#[cfg(not(debug_assertions))]
const DISCORD_TOKEN: &str = include_str!("../.token");

pub enum Event {
    Discord(DiscordEvent),
    Other
}

#[tokio::main]
async fn main() {
    // let java_exec = find_java_executable().unwrap();
    // extract_files().unwrap();

    // create http client and register commands
    let http_client = Arc::new(HttpClient::new(String::from(DISCORD_TOKEN.trim())));
    let application_id = http_client
        .current_user_application()
        .await
        .expect("receive current user application")
        .model()
        .await
        .expect("deserialize application data")
        .id;
    let interaction_client = http_client.interaction(application_id);
    set_commands(&interaction_client)
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

    let (event_sender, mut events) = tokio::sync::mpsc::channel::<(u64, Event)>(100);

    let mut events = futures::stream::select(
        discord_events.map(|item| (item.0, Event::Discord(item.1))),
        ReceiverStream::new(events)
    );
    while let Some((shard_id, event)) = events.next().await {
        if let Event::Discord(event) = &event {
            cache.update(event);
        }

        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http_client)));
    }
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<HttpClient>
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::Discord(DiscordEvent::ShardConnected(_)) => {
            println!("Connected on shard {shard_id}");
        }
        // Other events here...
        _ => {}
    }

    Ok(())
}
