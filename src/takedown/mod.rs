mod validation;

use std::time::Duration;

use dashmap::DashMap;
use rand::Rng;
use tokio::sync::oneshot;
use twilight_cache_inmemory::model::CachedVoiceState;
use twilight_cache_inmemory::Reference;
use twilight_lavalink::model::outgoing::{Play, Stop, Volume};
use twilight_model::application::interaction::Interaction;
use twilight_model::gateway::payload::outgoing::UpdateVoiceState;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::assets::replies;
use crate::sync::TaggedMutexGuard;
use crate::{Event, APPLICATION_ID, BOT_USER_ID, GUILD_VOICE_MUTEX, STATE};

const WULULULU_VOLUME: i64 = 40;

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
    Random,
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

    STATE
        .shard
        .command(&UpdateVoiceState::new(
            guild_id,
            Some(channel_id),
            true,
            false
        ))
        .await?;

    let player = STATE.lavalink.player(guild_id).await?;
    player.send(Volume::from((guild_id, WULULULU_VOLUME)))?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    player.send({
        let mut wululululu_play = Play::from((guild_id, &STATE.sounds.wululululu.0));
        wululululu_play.no_replace = true;
        wululululu_play
    })?;

    STATE.planned_takedowns.insert(guild_id, TakedownPlan {
        target_id,
        kind,
        guard
    });

    let factor = rand::thread_rng().gen_range(0.6..0.8);
    let play_duration = STATE.sounds.wululululu.1.mul_f64(factor);

    tokio::time::sleep(play_duration).await;
    STATE
        .event_sender
        .send(Event::TakedownStrike { guild_id })
        .await?;

    Ok(())
}

pub async fn handle_takedown_strike(guild_id: Id<GuildMarker>) -> anyhow::Result<()> {
    let Some((_, plan)) = STATE.planned_takedowns.remove(&guild_id) else {
        // someone else handled this
        return Ok(());
    };

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
    match (bot_channel_id, target_channel_id) {
        // at least one of the parties no longer in voice
        (None, _) | (_, None) => return Ok(()),

        // unequal channels, they fled or bot got moved
        (Some(b), Some(c)) if b != c => return Ok(()),

        _ => {}
    }

    let player = STATE.lavalink.player(guild_id).await?;
    player.send(Stop::new(guild_id))?;
    tokio::time::sleep(Duration::from_millis(200)).await;

    let vc_disconnect = STATE
        .http
        .update_guild_member(guild_id, target_id)
        .channel_id(None)
        .await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let sounds = &STATE.sounds;
    let sounds = match vc_disconnect {
        Ok(_) => &sounds.takedowns,
        Err(_) => &sounds.fails
    };
    let sound_index = rand::thread_rng().gen_range(0..sounds.len());
    let (track, length) = &sounds[sound_index];
    player.send(Play::from((guild_id, track)))?;

    tokio::time::sleep(*length + Duration::from_millis(100)).await;
    STATE
        .shard
        .command(&UpdateVoiceState::new(guild_id, None, false, false))
        .await?;

    if let TakedownType::User { resolve, .. } = kind {
        let _ = resolve.send(TakedownResolved::Success);
    }

    Ok(())
}
