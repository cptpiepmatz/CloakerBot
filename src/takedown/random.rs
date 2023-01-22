use std::cmp::Ordering;
use std::ops::Deref;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use rand::Rng;
use tokio::sync::oneshot;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::db::config::{GuildConfig, GuildConfigRandom};
use crate::db::random::GuildRandom;
use crate::db::DataStoreEntry;
use crate::log::LogMessage;
use crate::takedown::TakedownType;
use crate::{log, takedown, Event, STATE};

pub enum RandomTakedownExecution {
    Skipped,
    CarriedOut
}

pub async fn handle_random_check() -> anyhow::Result<()> {
    let entries = GuildRandom::fetch_all()
        .await?
        .into_iter()
        .filter(|d| d.next < DateTime::<Utc>::from(SystemTime::now()));

    let mut handles = Vec::new();
    for GuildRandom { id: guild_id, .. } in entries {
        handles.push(tokio::spawn(async move {
            handle_random_check_guild(guild_id).await.unwrap()
        }));
    }
    futures::future::join_all(handles).await;

    Ok(())
}

pub async fn handle_random_check_guild(guild_id: Id<GuildMarker>) -> anyhow::Result<()> {
    let config = GuildConfig::fetch(guild_id).await?;
    if config.random.enabled {
        let res = execute_random_takedown(guild_id).await?;

        let next = fetch_next_check(guild_id).await?;
        let mut random = GuildRandom::fetch(guild_id).await?;
        random.next = next;
        random.store(guild_id).await?;

        if let RandomTakedownExecution::Skipped = res {
            if let Some(channel_id) = log::fetch_log_channel(guild_id).await? {
                log::log_message(guild_id, channel_id, LogMessage::TakedownCheckNoStrike {
                    next
                })
                .await?;
            }
        }
    }

    Ok(())
}

pub async fn execute_random_takedown(
    guild_id: Id<GuildMarker>
) -> anyhow::Result<RandomTakedownExecution> {
    let Some(voice_states) = STATE
        .event_cache
        .guild_voice_states(guild_id) else {
        return Ok(RandomTakedownExecution::Skipped);
    };
    let voice_states = voice_states.deref();

    if voice_states.len() < 1 {
        return Ok(RandomTakedownExecution::Skipped);
    }

    let target_id = {
        let target_index = rand::thread_rng().gen_range(0..voice_states.len());
        *voice_states.iter().nth(target_index).expect("in range")
    };

    if !takedown::validation::user_is_valid_target(guild_id, target_id, true).await? {
        return Ok(RandomTakedownExecution::Skipped);
    }

    let (tx, rx) = oneshot::channel();
    STATE
        .event_sender
        .send(Event::TakedownInit {
            target_id,
            guild_id,
            kind: TakedownType::Random { resolve: tx }
        })
        .await?;

    let _resolved = rx.await?;
    Ok(RandomTakedownExecution::CarriedOut)
}

pub async fn fetch_next_check(guild_id: Id<GuildMarker>) -> anyhow::Result<DateTime<Utc>> {
    let config = GuildConfig::fetch(guild_id).await?;
    let GuildConfigRandom { min, max, .. } = config.random;
    let [min, max] = match min.total_cmp(&max) {
        Ordering::Less | Ordering::Equal => [min, max],
        Ordering::Greater => [max, min]
    };
    let offset = rand::thread_rng().gen_range(min..=max);
    Ok(
        DateTime::from(SystemTime::now()) +
            chrono::Duration::seconds((60.0 * 60.0 * offset) as i64)
    )
}
