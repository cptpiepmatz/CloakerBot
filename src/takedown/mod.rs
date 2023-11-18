use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use derive_more::From;
use rand::prelude::*;
use songbird::error::JoinError;
use songbird::Call;
use tokio::sync::{Mutex, MutexGuard};
use twilight_cache_inmemory::model::CachedVoiceState;
use twilight_cache_inmemory::Reference;
use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_http::error::ErrorType;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::config::GuildConfig;
use crate::sync::VolatileMapGuard;
use crate::takedown::sounds::Sound;
use crate::util::ClientError;
use crate::{util, BOT_USER_ID, STATE};

pub mod counter;
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
        by_id: Id<UserMarker>
    },
    // TODO: maybe this needs a variant for other discord-related errors
    Join(JoinError),

    // Any error that could happen when interacting with discord via their http interface.
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
    /// If countered, the user that countered will be placed here.
    countered_by_id: Option<Id<UserMarker>>
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
    // TODO: split this function into multiple
    // TODO: put this into inner function to allow tracking stats

    let rng = &mut rand::rngs::StdRng::from_entropy();
    let guild_configs = STATE.guild_configs.read().await;
    let config = GuildConfig::default();
    let config = guild_configs.get(&guild_id).unwrap_or(&config);
    let target_roles = fetch_target_roles(guild_id, target_id).await?;
    preliminary_checks(kind, config, &target_roles)?;
    let channel_id = get_channel_id(target_id, guild_id)?;

    let _takedown_lock = STATE.takedown_locks.lock(guild_id);
    let _planned_takedown_guard = upsert_takedown_plan(target_id, guild_id);

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
        rng
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
    rng: &mut impl Rng
) -> Result<TakedownSuccess, TakedownFail> {
    let (notification_sound, takedown_sound, fail_sound) = choose_sounds(rng);

    let notification_track_handle =
        call.play_only_input(notification_sound.0.input.new_handle().into());
    tokio::time::sleep(sounds::WULULULULU.duration.mul_f64(notification_sound.1)).await;

    notification_track_handle
        .stop()
        .expect("track is owned here");

    // while sleeping here, a counter could occur
    if let TakedownType::Called { caller_id } = kind {
        if let Some(countered_by_id) = STATE
            .planned_takedowns
            .apply(guild_id, |plan| plan.countered_by_id)
            .expect("guard for that entry still alive")
        {
            let error = Err(TakedownFail::Countered {
                by_id: countered_by_id
            });

            // do the counter measures here
            let Some(caller_id) = caller_id.into()
            else {
                // no caller_id, bot did it own it's own
                call.play_only_input(fail_sound.input.new_handle().into());
                tokio::time::sleep(fail_sound.duration + WAIT_AFTER_QUOTE).await;
                return error;
            };

            let caller_is_here = match STATE.cache.voice_state(caller_id, guild_id) {
                None => false,
                Some(caller_vs) => caller_vs.channel_id() == channel_id
            };

            let counter_is_immune =
                if let Some(counter) = STATE.cache.member(guild_id, countered_by_id) {
                    config
                        .immunity
                        .general
                        .iter()
                        .chain(config.immunity.general_silent.iter())
                        .any(|role| counter.roles().contains(role))
                }
                else {
                    true
                };

            return match caller_is_here && !counter_is_immune {
                true => {
                    if let Err(err) = STATE
                        .client
                        .update_guild_member(guild_id, caller_id)
                        .channel_id(None)
                        .await
                    {
                        // TODO: check if this code is the missing permissions code
                        if matches!(err.kind(), ErrorType::Response {
                            error: ApiError::General(GeneralApiError {
                                code: MISSING_PERMS_ERROR_CODE,
                                ..
                            }),
                            ..
                        }) {
                            fail_sound.play_and_wait(&mut call, None).await;
                            return error;
                        }

                        return Err(TakedownFail::Client(err.into()));
                    }

                    takedown_sound
                        .play_and_wait(&mut call, WAIT_AFTER_QUOTE)
                        .await;
                    error
                }
                false => {
                    fail_sound.play_and_wait(&mut call, None).await;
                    error
                }
            };
        }
    }

    // TODO: check for fleeing

    // no counter, continue with takedown
    let has_immune_role = target_roles
        .iter()
        .any(|role| config.immunity.general.contains(role));
    if has_immune_role {
        fail_sound.play_and_wait(&mut call, None).await;
        return Err(TakedownFail::Immune);
    }

    if let Err(err) = STATE
        .client
        .update_guild_member(guild_id, target_id)
        .channel_id(None)
        .await
    {
        if matches!(err.kind(), ErrorType::Response {
            error: ApiError::General(GeneralApiError {
                code: MISSING_PERMS_ERROR_CODE,
                ..
            }),
            ..
        }) {
            fail_sound.play_and_wait(&mut call, None).await;
            return Err(TakedownFail::MissingPermissions);
        }

        return Err(TakedownFail::Client(err.into()));
    }

    takedown_sound
        .play_and_wait(&mut call, WAIT_AFTER_QUOTE)
        .await;
    Ok(TakedownSuccess)
}

#[inline]
async fn fetch_target_roles(
    guild_id: Id<GuildMarker>,
    target_id: Id<UserMarker>
) -> Result<Vec<Id<RoleMarker>>, TakedownFail> {
    match util::cache_or_fetch(
        STATE.cache.member(guild_id, target_id),
        |member| member.roles().to_vec(),
        STATE.client.guild_member(guild_id, target_id),
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
        if member_roles
            .iter()
            .any(|role| config.immunity.random_silent.contains(role))
        {
            return Err(TakedownFail::Immune);
        }
    }

    if member_roles
        .iter()
        .any(|role| config.immunity.general_silent.contains(role))
    {
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
    guild_id: Id<GuildMarker>
) -> VolatileMapGuard<'static, Id<GuildMarker>, PlannedTakedown> {
    STATE
        .planned_takedowns
        .upsert(guild_id, PlannedTakedown {
            target_id,
            countered_by_id: None
        })
        .expect("is unset as per guild this is a critical section")
}

#[inline]
async fn get_call(
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>
) -> Result<Arc<Mutex<Call>>, TakedownFail> {
    Ok(STATE
        .songbird
        .join(guild_id, channel_id)
        .await
        .map_err(TakedownFail::Join)?)
}

#[inline]
fn choose_sounds(rng: &mut impl Rng) -> ((&Sound, f64), &Sound, &Sound) {
    let notification_sound = sounds::WULULULULU.deref();
    let duration_multiplier = rng.gen_range(0.3..0.8);

    let takedown_sound = sounds::TAKEDOWNS
        .choose(rng)
        .expect("takedown sounds are not empty");

    let fail_sound = sounds::FAILS
        .choose(rng)
        .expect("fail sounds are not empty");

    (
        (notification_sound, duration_multiplier),
        takedown_sound,
        fail_sound
    )
}
