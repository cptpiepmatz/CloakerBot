pub mod random;
pub mod validation;

use std::time::Duration;

use dashmap::DashMap;
use rand::Rng;
use songbird::tracks::Track;
use tokio::sync::oneshot;
use twilight_cache_inmemory::model::CachedVoiceState;
use twilight_cache_inmemory::Reference;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::log::LogMessage;
use crate::sync::TaggedMutexGuard;
use crate::{log, Event, BOT_USER_ID, GUILD_VOICE_MUTEX, STATE};

const WULULULU_VOLUME: f32 = 0.3;
const TAKEDOWN_VOLUME: f32 = 0.5;
const FAIL_VOLUME: f32 = 1.0;

// the tagged mutex guards implicitly this map
// only one takedown may be initialized at the same type per guild,
// therefore only one plan can be inserted into the map per guild
pub type PlannedTakedowns = DashMap<Id<GuildMarker>, TakedownPlan>;

#[derive(Debug)]
pub struct TakedownPlan {
    target_id: Id<UserMarker>,
    kind: TakedownType,
    guard: TaggedMutexGuard<'static, Id<GuildMarker>>
}

#[derive(Debug)]
pub enum TakedownType {
    User {
        initiator_id: Id<UserMarker>,
        resolve: oneshot::Sender<TakedownResolved>
    },
    Random {
        resolve: oneshot::Sender<TakedownResolved>
    },
    DangerZone
}

#[derive(Debug)]
pub enum TakedownResolved {
    Success,
    Fail,
    Counter { by: Id<UserMarker> }
}

pub async fn handle_takedown_init(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    kind: TakedownType
) -> anyhow::Result<()> {
    let guard = GUILD_VOICE_MUTEX.lock(guild_id).await;

    let event_cache = &STATE.event_cache;
    let Some(voice_state) = event_cache.voice_state(target_id, guild_id) else {
        // no known voice state, nothing we can do about it
        return Ok(());
    };
    let channel_id = voice_state.channel_id();

    let call = STATE.songbird.join(guild_id, channel_id).await?;
    let mut call = call.lock().await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let wulululu = Track::new(STATE.sounds.wululululu.data.clone().into()).volume(WULULULU_VOLUME);
    call.play_only(wulululu);

    STATE.planned_takedowns.insert(guild_id, TakedownPlan {
        target_id,
        kind,
        guard
    });

    let factor = rand::thread_rng().gen_range(0.6..0.8);
    let play_duration = STATE.sounds.wululululu.duration.mul_f64(factor);

    tokio::time::sleep(play_duration).await;
    if (STATE
        .event_sender
        .send(Event::TakedownStrike { guild_id })
        .await)
        .is_err()
    {
        call.leave().await?;
    }

    Ok(())
}

pub async fn handle_takedown_strike(guild_id: Id<GuildMarker>) -> anyhow::Result<()> {
    let Some((_, plan)) = STATE.planned_takedowns.remove(&guild_id) else {
        // someone else handled this
        return Ok(());
    };

    let Some(call) = STATE.songbird.get(guild_id) else {
        // no call available, we're out
        return Ok(())
    };
    let mut call = call.lock().await;

    let TakedownPlan {
        target_id,
        guard: _guard,
        kind
    } = plan;

    let channel_id_from_cache = |c_ref: Reference<_, CachedVoiceState>| c_ref.channel_id();
    let bot_channel_id = STATE
        .event_cache
        .voice_state(BOT_USER_ID, guild_id)
        .map(channel_id_from_cache);
    let target_channel_id = STATE
        .event_cache
        .voice_state(target_id, guild_id)
        .map(channel_id_from_cache);

    call.stop();
    tokio::time::sleep(Duration::from_millis(200)).await;

    let takedown_successful = match (bot_channel_id, target_channel_id) {
        // bot no longer in channel
        (None, _) => return Ok(()),

        // target fled
        (_, None) => false,

        // unequal channels, they fled or bot got moved
        (Some(b), Some(t)) if b != t => false,

        (Some(_), Some(_)) => STATE
            .http
            .update_guild_member(guild_id, target_id)
            .channel_id(None)
            .await
            .is_ok()
    };

    tokio::time::sleep(Duration::from_millis(500)).await;

    let (sound, volume_factor) = match takedown_successful {
        true => (STATE.sounds.select_takedown(), TAKEDOWN_VOLUME),
        false => (STATE.sounds.select_fail(), FAIL_VOLUME)
    };
    let track = Track::new(sound.data.clone().into()).volume(volume_factor);
    call.play_only(track);

    tokio::time::sleep(sound.duration + Duration::from_millis(100)).await;
    call.leave().await?;

    let resolved = match takedown_successful {
        true => TakedownResolved::Success,
        false => TakedownResolved::Fail
    };
    match kind {
        TakedownType::User {
            initiator_id,
            resolve
        } => {
            let _ = resolve.send(resolved);
            if let (Some(channel_id), true) =
                (log::fetch_log_channel(guild_id).await?, takedown_successful)
            {
                log::log_message(guild_id, channel_id, LogMessage::UserTakedownStrike {
                    target_id,
                    initiator_id
                })
                .await?;
            }
        }
        TakedownType::Random { resolve } => {
            let _ = resolve.send(resolved);
            if let (Some(channel_id), true) =
                (log::fetch_log_channel(guild_id).await?, takedown_successful)
            {
                log::log_message(guild_id, channel_id, LogMessage::RandomTakedownStrike {
                    target_id,
                    next: random::fetch_next_check(guild_id).await?
                })
                .await?;
            }
        }
        TakedownType::DangerZone => {
            if let Some(channel_id) = log::fetch_log_channel(guild_id).await? {
                log::log_message(guild_id, channel_id, LogMessage::DangerZoneTakedownStrike {
                    target_id,
                    channel_id
                })
                .await?;
            }
        }
    }

    Ok(())
}
