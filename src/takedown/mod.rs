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
use crate::discord::error_codes::{MISSING_ACCESS, MISSING_PERMISSIONS};
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
    Database(sqlx::Error),
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

fn missing_permissions(error: &twilight_http::Error) -> bool {
    matches!(error.kind(), ErrorType::Response {
        error: ApiError::General(GeneralApiError {
            code: MISSING_PERMISSIONS | MISSING_ACCESS,
            ..
        }),
        ..
    })
}

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

type TakedownPlan = (
    VolatileMapGuard<'static, Id<GuildMarker>, PlannedTakedown>,
    Option<oneshot::Receiver<Id<UserMarker>>>
);

struct VoiceTakedown<'a> {
    target_id: Id<UserMarker>,
    target_roles: &'a [Id<RoleMarker>],
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    kind: TakedownType,
    config: &'a GuildConfig
}

#[derive(Debug, Clone, Copy)]
pub enum TakedownType {
    Called { caller_id: Id<UserMarker> },
    Random,
    DangerZone { channel_id: Id<ChannelMarker> }
}

#[tracing::instrument(skip_all, fields(%guild_id, %target_id, ?kind))]
pub async fn takedown(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    kind: TakedownType
) -> TakedownResult {
    tracing::debug!("waiting for takedown lock");
    let _takedown_lock = STATE.takedown_locks.lock(guild_id).await;
    tracing::debug!("acquired takedown lock");
    if STATE.cache.guild(guild_id).is_none() {
        tracing::info!("takedown did not complete: guild not found");
        return Err(TakedownFail::NotFound);
    }
    let config = STATE.database.guild_config(guild_id).await.map_err(|error| {
        tracing::error!(?error, "could not load takedown configuration");
        TakedownFail::Database(error)
    })?;
    let mut channel_id = None;
    let result = execute_takedown(target_id, guild_id, kind, &config, &mut channel_id).await;
    record_outcome(target_id, guild_id, channel_id, kind, &result).await;
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
    channel_id: Option<Id<ChannelMarker>>,
    kind: TakedownType,
    result: &TakedownResult
) {
    use crate::data::stats::StatEvent;
    let event = match result {
        Ok(_) => {
            let caller_id = match kind {
                TakedownType::Called { caller_id } => caller_id,
                TakedownType::Random | TakedownType::DangerZone { .. } => BOT_USER_ID
            };
            Some(StatEvent::Takedown { caller_id, target_id })
        }
        Err(TakedownFail::Countered { by_id, outcome }) => match outcome {
            CounterOutcome::Parried => Some(StatEvent::Parry { user_id: *by_id }),
            CounterOutcome::Removed { user_id } => {
                Some(StatEvent::Counter { caller_id: *by_id, target_id: *user_id })
            }
        },
        Err(_) => None
    };
    if let Some(event) = event &&
        let Err(error) = STATE.database.record_stat(guild_id, event).await
    {
        tracing::error!(%guild_id, ?error, "could not persist completed takedown");
    }
    if let Err(error) = result {
        tracing::info!(%guild_id, %target_id, ?kind, ?error, "takedown did not complete");
    }
    else {
        tracing::info!(%guild_id, %target_id, ?kind, "takedown completed");
    }
    if let Err(error) =
        crate::discord::channel_logging::log_takedown(guild_id, target_id, channel_id, kind, result)
            .await
    {
        tracing::warn!(%guild_id, ?error, "could not log takedown to discord");
    }
}

async fn execute_takedown(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    kind: TakedownType,
    config: &GuildConfig,
    target_channel: &mut Option<Id<ChannelMarker>>
) -> TakedownResult {
    let rng = &mut rand::rngs::StdRng::from_rng(&mut rand::rng());
    if !config.target_bots {
        let is_bot = util::cache_or_fetch(
            STATE.cache.user(target_id),
            |user| user.bot,
            STATE.client.user(target_id),
            |user| user.bot
        )
        .await
        .map_err(TakedownFail::Client)?;
        if is_bot {
            return Err(TakedownFail::Immune);
        }
    }
    let target_roles = fetch_member_roles(guild_id, target_id).await?;
    preliminary_checks(kind, config, &target_roles)?;
    let channel_id = get_channel_id(target_id, guild_id)?;
    *target_channel = Some(channel_id);
    if let TakedownType::DangerZone { channel_id: expected } = kind &&
        channel_id != expected
    {
        return Err(TakedownFail::Fled);
    }

    let (_planned_takedown_guard, counter_rx) =
        upsert_takedown_plan(target_id, guild_id, !matches!(kind, TakedownType::DangerZone { .. }));

    tracing::debug!(%channel_id, "joining voice channel");
    let call = get_call(guild_id, channel_id).await?;
    tracing::debug!(%channel_id, "joined voice channel; waiting for call lock");
    let mut call = call.lock().await;
    tracing::debug!(%channel_id, "acquired voice call lock");

    let takedown_res = takedown_in_voice(
        &mut call,
        VoiceTakedown {
            target_id,
            target_roles: &target_roles,
            guild_id,
            channel_id,
            kind,
            config
        },
        rng,
        counter_rx
    )
    .await;

    // leave the channel after everything that happened
    // nothing we can do if leaving fails
    tracing::debug!(%channel_id, "leaving voice channel");
    if let Err(error) = call.leave().await {
        tracing::warn!(%channel_id, ?error, "could not leave voice channel");
    }
    takedown_res
}

#[inline]
async fn takedown_in_voice(
    call: &mut MutexGuard<'_, Call>,
    takedown: VoiceTakedown<'_>,
    rng: &mut impl Rng,
    counter_rx: Option<oneshot::Receiver<Id<UserMarker>>>
) -> Result<TakedownSuccess, TakedownFail> {
    let VoiceTakedown { target_id, target_roles, guild_id, channel_id, kind, config } = takedown;
    let (notification_sound, takedown_sound, fail_sound) = choose_sounds(rng);
    tracing::debug!(%channel_id, "playing takedown notification");

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
    tracing::debug!(%channel_id, countered_by_id = countered_by_id.map(|id| id.get()), "takedown notification finished");

    let bot_channel =
        STATE.cache.voice_state(BOT_USER_ID, guild_id).map(|state| state.channel_id());
    if bot_channel != Some(channel_id) ||
        call.current_channel() != Some(ChannelId::from(channel_id))
    {
        return Err(TakedownFail::Rejected);
    }

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

        if !caller_is_here {
            fail_sound.play_and_wait(call, None).await;
            return fail;
        }
        let caller_roles = match fetch_member_roles(guild_id, caller_id).await {
            Ok(roles) => roles,
            Err(error) => {
                tracing::warn!(%guild_id, %caller_id, ?error, "could not check counter target");
                fail_sound.play_and_wait(call, None).await;
                return fail;
            }
        };
        let caller_is_immune = config
            .immunity
            .general
            .iter()
            .chain(config.immunity.general_silent.iter())
            .any(|role| caller_roles.contains(role));

        if caller_is_immune {
            fail_sound.play_and_wait(call, None).await;
            return fail;
        }

        if let Err(err) =
            STATE.client.update_guild_member(guild_id, caller_id).channel_id(None).await
        {
            if !missing_permissions(&err) {
                tracing::warn!(%guild_id, %caller_id, error = ?err, "returned takedown failed");
            }
            fail_sound.play_and_wait(call, None).await;
            return fail;
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

    // no counter so continue with takedown
    let has_immune_role = target_roles.iter().any(|role| {
        config.immunity.general.contains(role) ||
            (matches!(kind, TakedownType::Random) && config.immunity.random.contains(role))
    });
    if has_immune_role {
        fail_sound.play_and_wait(call, None).await;
        return Err(TakedownFail::Immune);
    }

    if let Err(err) = STATE.client.update_guild_member(guild_id, target_id).channel_id(None).await {
        if missing_permissions(&err) {
            fail_sound.play_and_wait(call, None).await;
            return Err(TakedownFail::MissingPermissions);
        }

        return Err(TakedownFail::Client(err.into()));
    }

    takedown_sound.play_and_wait(call, WAIT_AFTER_QUOTE).await;
    Ok(TakedownSuccess)
}

#[inline]
async fn fetch_member_roles(
    guild_id: Id<GuildMarker>,
    member_id: Id<UserMarker>
) -> Result<Vec<Id<RoleMarker>>, TakedownFail> {
    let member = STATE
        .client
        .guild_member(guild_id, member_id)
        .await
        .map_err(ClientError::from)?
        .model()
        .await
        .map_err(ClientError::from)?;
    Ok(member.roles)
}

#[inline]
fn preliminary_checks(
    kind: TakedownType,
    config: &GuildConfig,
    member_roles: &[Id<RoleMarker>]
) -> Result<(), TakedownFail> {
    if let TakedownType::Random = kind &&
        member_roles.iter().any(|role| config.immunity.random_silent.contains(role))
    {
        return Err(TakedownFail::Immune);
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
) -> TakedownPlan {
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
    let call = STATE.songbird.get_or_insert(guild_id);
    let result = async {
        call.lock().await.deafen(true).await.map_err(TakedownFail::Join)?;
        STATE.songbird.join(guild_id, channel_id).await.map_err(TakedownFail::Join)
    }
    .await;
    // a failed join may already have moved the bot into the channel
    if result.is_err() &&
        let Err(error) = call.lock().await.leave().await
    {
        tracing::warn!(%guild_id, %channel_id, ?error, "could not leave voice channel after failed join");
    }
    result
}

#[inline]
fn choose_sounds(rng: &mut impl Rng) -> ((&Sound, f64), &Sound, &Sound) {
    let notification_sound = sounds::WULULULULU.deref();
    let duration_multiplier = rng.random_range(0.3..0.8);

    let takedown_sound = sounds::TAKEDOWNS.choose(rng).expect("takedown sounds are not empty");

    let fail_sound = sounds::FAILS.choose(rng).expect("fail sounds are not empty");

    ((notification_sound, duration_multiplier), takedown_sound, fail_sound)
}
