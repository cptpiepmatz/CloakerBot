use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use derive_more::From;
use rand::prelude::*;
use songbird::Call;
use songbird::error::JoinError;
use songbird::id::ChannelId;
use tokio::select;
use tokio::sync::{Mutex, MutexGuard, oneshot};
use twilight_cache_inmemory::Reference;
use twilight_cache_inmemory::model::CachedVoiceState;
use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_http::error::ErrorType;
use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};

use crate::data::config::GuildConfig;
use crate::sync::VolatileMapGuard;
use crate::takedown::sounds::Sound;
use crate::util::ClientError;
use crate::{BOT_USER_ID, STATE, util};

pub mod counter;
pub mod danger_zones;
pub mod random;
pub mod sounds;

const WAIT_AFTER_QUOTE: Duration = Duration::from_millis(300);

pub type TakedownResult = Result<TakedownSuccess, TakedownFail>;

pub struct TakedownSuccess;

#[derive(Debug, From)]
pub enum TakedownFail {
    /// The takedown was countered.
    ///
    /// `by_id` refers to the user who countered the bot.
    /// It may be another user than the target.
    Countered {
        by_id: Id<UserMarker>,
        outcome: CounterOutcome
    },
    Join(JoinError),
    /// Audio playback control failed.
    Audio(songbird::tracks::ControlError),
    /// An HTTP request or response-body decoding failed.
    Client(ClientError),
    /// Missing permissions to either join the channel or kick the target.
    MissingPermissions,
    /// Could not find the user.
    ///
    /// The channel could be not visible to the bot or the user is currently not
    /// a channel on that guild.
    NotFound,
    /// The target fled before the bot could finish its takedown.
    Fled,
    /// The target is immune.
    Immune,
    /// The bot got removed from the voice channel.
    Rejected
}

const MISSING_PERMS_ERROR_CODE: u64 = 50013;

/// Takedown plan.
///
/// Applied to a hash map with guilds as keys.
/// Some values are only for debug purposes.
#[derive(Debug)]
pub struct PlannedTakedown {
    /// Target of the takedown.
    target_id: Id<UserMarker>,
    /// Sender for the counter oneshot channel.
    counter_tx: Option<oneshot::Sender<Id<UserMarker>>>
}

#[derive(Debug, Clone, Copy)]
pub enum TakedownType {
    Called { caller_id: Id<UserMarker> },
    Random,
    DangerZone { channel_id: Id<ChannelMarker> }
}

pub async fn takedown(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    kind: TakedownType
) -> TakedownResult {
    let guild_configs = STATE.guild_configs.read().await;
    let config = guild_configs.get(&guild_id).cloned().unwrap_or_default();
    drop(guild_configs);
    let result = execute_takedown(target_id, guild_id, kind, &config).await;
    record_outcome(target_id, guild_id, kind, &result).await;
    result
}

#[derive(Debug, Clone, Copy)]
pub enum CounterOutcome {
    Parried,
    Removed { user_id: Id<UserMarker> }
}

async fn record_outcome(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    kind: TakedownType,
    result: &TakedownResult
) {
    match result {
        Ok(_) => {
            let caller_id = match kind {
                TakedownType::Called { caller_id } => caller_id,
                TakedownType::Random | TakedownType::DangerZone { .. } => BOT_USER_ID
            };
            STATE.stats.write().await.takedowns.entry(guild_id).or_default()
                .increment_takedown(caller_id, target_id);
        }
        Err(TakedownFail::Countered { by_id, outcome }) => {
            let mut stats = STATE.stats.write().await;
            let stats = stats.takedowns.entry(guild_id).or_default();
            match outcome {
                CounterOutcome::Parried => { stats.increment_parry(*by_id); }
                CounterOutcome::Removed { user_id } => { stats.increment_counter(*by_id, *user_id); }
            }
        }
        Err(_) => {}
    }
    if let Err(error) = result {
        macros::info!("takedown in {guild_id} targeting {target_id} ({kind:?}): {error:?}");
    }
}

async fn execute_takedown(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    kind: TakedownType,
    config: &GuildConfig
) -> TakedownResult {
    let rng = &mut rand::rngs::StdRng::from_os_rng();
    let _takedown_lock = STATE.takedown_locks.lock(guild_id).await;
    let target_roles = fetch_member_roles(guild_id, target_id).await?;
    preliminary_checks(kind, config, &target_roles)?;
    let channel_id = get_channel_id(target_id, guild_id)?;

    let (_planned_takedown_guard, counter_rx) =
        upsert_takedown_plan(target_id, guild_id, !matches!(kind, TakedownType::DangerZone { .. }));

    let call = get_call(guild_id, channel_id).await?;
    let mut call = call.lock().await;

    let takedown_res = takedown_in_voice(
        &mut call,
        target_id,
        &target_roles,
        guild_id,
        channel_id,
        kind,
        config,
        rng,
        counter_rx
    )
    .await;


    // leave the channel after everything that happened, we cannot do a thing if it
    // fails to leave
    let _ = call.leave().await;
    takedown_res
}

#[inline]
async fn takedown_in_voice(
    mut call: &mut MutexGuard<'_, Call>,
    target_id: Id<UserMarker>,
    target_roles: &[Id<RoleMarker>],
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    kind: TakedownType,
    config: &GuildConfig,
    rng: &mut impl Rng,
    counter_rx: Option<oneshot::Receiver<Id<UserMarker>>>
) -> Result<TakedownSuccess, TakedownFail> {
    let (notification_sound, takedown_sound, fail_sound) = choose_sounds(rng);

    let notification_track_handle =
        call.play_only_input(notification_sound.0.input.new_handle().into());
    let notification_sleep =
        tokio::time::sleep(sounds::WULULULULU.duration.mul_f64(notification_sound.1));

    // while sleeping here, a counter could occur
    let countered_by_id: Option<Id<UserMarker>> = match counter_rx {
        Some(counter_rx) => select! {
            counter_res = counter_rx => counter_res.ok(),
            _ = notification_sleep => None
        },
        None => {
            notification_sleep.await;
            None
        }
    };

    notification_track_handle.stop().map_err(TakedownFail::Audio)?;

    let caller_id = match kind {
        TakedownType::Called { caller_id } => Some(caller_id),
        TakedownType::Random => None,
        TakedownType::DangerZone { .. } => None
    };

    if let Some(countered_by_id) = countered_by_id {
        let fail = Err(TakedownFail::Countered {
            by_id: countered_by_id,
            outcome: CounterOutcome::Parried
        });

        let Some(caller_id) = caller_id
        else {
            fail_sound.play_and_wait(call, WAIT_AFTER_QUOTE).await;
            return fail;
        };

        let caller_is_here = match STATE.cache.voice_state(caller_id, guild_id) {
            Some(caller_vs) => caller_vs.channel_id() == channel_id,
            None => false
        };

        let caller_roles = fetch_member_roles(guild_id, caller_id).await?;
        let caller_is_immune = config
            .immunity
            .general
            .iter()
            .chain(config.immunity.general_silent.iter())
            .any(|role| caller_roles.contains(role));

        if caller_is_immune || !caller_is_here {
            fail_sound.play_and_wait(&mut call, None).await;
            return fail;
        }

        if let Err(err) =
            STATE.client.update_guild_member(guild_id, caller_id).channel_id(None).await
        {
            // TODO: check if this code is the missing permissions code
            if matches!(err.kind(), ErrorType::Response {
                error: ApiError::General(GeneralApiError { code: MISSING_PERMS_ERROR_CODE, .. }),
                ..
            }) {
                fail_sound.play_and_wait(&mut call, None).await;
                return fail;
            }

            return Err(TakedownFail::Client(err.into()));
        }

        takedown_sound.play_and_wait(call, WAIT_AFTER_QUOTE).await;
        return Err(TakedownFail::Countered {
            by_id: countered_by_id,
            outcome: CounterOutcome::Removed { user_id: caller_id }
        });
    }

    let target_end_channel_id =
        STATE.cache.voice_state(target_id, guild_id).as_ref().map(|cvs| cvs.channel_id());
    if target_end_channel_id != channel_id.into() {
        fail_sound.play_and_wait(call, None).await;
        return Err(TakedownFail::Fled);
    }

    // TODO: check for rejection, check if this works now
    if call.current_channel() != ChannelId::from(channel_id).into() {
        fail_sound.play_and_wait(call, None).await;
        return Err(TakedownFail::Rejected);
    }

    // no counter, continue with takedown
    let has_immune_role = target_roles.iter().any(|role| config.immunity.general.contains(role));
    if has_immune_role {
        fail_sound.play_and_wait(&mut call, None).await;
        return Err(TakedownFail::Immune);
    }

    if let Err(err) = STATE.client.update_guild_member(guild_id, target_id).channel_id(None).await {
        if matches!(err.kind(), ErrorType::Response {
            error: ApiError::General(GeneralApiError { code: MISSING_PERMS_ERROR_CODE, .. }),
            ..
        }) {
            fail_sound.play_and_wait(&mut call, None).await;
            return Err(TakedownFail::MissingPermissions);
        }

        return Err(TakedownFail::Client(err.into()));
    }

    takedown_sound.play_and_wait(&mut call, WAIT_AFTER_QUOTE).await;
    Ok(TakedownSuccess)
}

#[inline]
async fn fetch_member_roles(
    guild_id: Id<GuildMarker>,
    member_id: Id<UserMarker>
) -> Result<Vec<Id<RoleMarker>>, TakedownFail> {
    match util::cache_or_fetch(
        STATE.cache.member(guild_id, member_id),
        |member| member.roles().to_vec(),
        STATE.client.guild_member(guild_id, member_id),
        |member| member.roles
    )
    .await
    {
        Ok(member_roles) => Ok(member_roles),
        Err(err) => Err(TakedownFail::Client(err))
    }
}

#[inline]
fn preliminary_checks(
    kind: TakedownType,
    config: &GuildConfig,
    member_roles: &[Id<RoleMarker>]
) -> Result<(), TakedownFail> {
    if let TakedownType::Random = kind {
        if member_roles.iter().any(|role| config.immunity.random_silent.contains(role)) {
            return Err(TakedownFail::Immune);
        }
    }

    if member_roles.iter().any(|role| config.immunity.general_silent.contains(role)) {
        return Err(TakedownFail::Immune);
    }

    Ok(())
}

#[inline]
fn get_channel_id(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>
) -> Result<Id<ChannelMarker>, TakedownFail> {
    STATE
        .cache
        .voice_state(target_id, guild_id)
        .map(|cvs| cvs.channel_id())
        .ok_or(TakedownFail::NotFound)
}

#[inline]
fn upsert_takedown_plan(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    allow_counter: bool
) -> (
    VolatileMapGuard<'static, Id<GuildMarker>, PlannedTakedown>,
    Option<oneshot::Receiver<Id<UserMarker>>>
) {
    let (tx, rx) = match (allow_counter, oneshot::channel()) {
        (true, (tx, rx)) => (Some(tx), Some(rx)),
        (false, _) => (None, None)
    };

    (
        STATE
            .planned_takedowns
            .upsert(guild_id, PlannedTakedown { target_id, counter_tx: tx })
            .expect("is unset as per guild this is a critical section"),
        rx
    )
}

#[inline]
async fn get_call(
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>
) -> Result<Arc<Mutex<Call>>, TakedownFail> {
    Ok(STATE.songbird.join(guild_id, channel_id).await.map_err(TakedownFail::Join)?)
}

#[inline]
fn choose_sounds(rng: &mut impl Rng) -> ((&Sound, f64), &Sound, &Sound) {
    let notification_sound = sounds::WULULULULU.deref();
    let duration_multiplier = rng.random_range(0.3..0.8);

    let takedown_sound = sounds::TAKEDOWNS.choose(rng).expect("takedown sounds are not empty");

    let fail_sound = sounds::FAILS.choose(rng).expect("fail sounds are not empty");

    ((notification_sound, duration_multiplier), takedown_sound, fail_sound)
}
