use std::ops::Deref;
use std::time::Duration;

use derive_more::From;
use rand::prelude::*;
use songbird::error::JoinError;
use twilight_cache_inmemory::model::CachedVoiceState;
use twilight_cache_inmemory::Reference;
use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_http::error::ErrorType;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::config::GuildConfig;
use crate::{BOT_USER_ID, STATE};

pub mod sounds;

const WAIT_AFTER_QUOTE: Duration = Duration::from_secs(1);

// used to provide a response to the caller
#[derive(Debug, From)]
pub enum TakedownResult {
    /// The takedown was successful.
    Success,
    /// The takedown failed.
    Fail(TakedownFail),
    /// The takedown was countered.
    ///
    /// `by_id` refers to the user who countered the bot.
    /// It may be another user than the target.
    Countered { by_id: Id<UserMarker> }
}

#[derive(Debug, From)]
pub enum TakedownFail {
    // TODO: maybe this needs a variant for other discord-related errors
    Join(JoinError),

    // Any error that could happen when interacting with discord via their http interface.
    HttpError(twilight_http::error::Error),
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
    _target_id: Id<UserMarker>,
    _channel_id: Id<ChannelMarker>,
    /// Caller of the takedown plan.
    ///
    /// If [`None`], the bot itself invoked this.
    _caller_id: Option<Id<UserMarker>>,
    /// If countered, the user that countered will be placed here.
    countered_by_id: Option<Id<UserMarker>>
}

pub async fn takedown(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    caller_id: impl Into<Option<Id<UserMarker>>> + Copy,
    random: bool
) -> TakedownResult {
    // TODO: split this function into multiple
    // TODO: put this into inner function to allow tracking stats

    let rng = &mut rand::rngs::StdRng::from_entropy();
    let guild_configs = STATE.guild_configs.read().await;
    let config = GuildConfig::default();
    let config = guild_configs.get(&guild_id).unwrap_or(&config);

    // if the user is not part of the cache the bot will most likely not know
    // where the user is
    let Some(user_target) = STATE.cache.user(target_id)
    else {
        return TakedownFail::NotFound.into();
    };
    let user_target = user_target.deref();

    // if targeting bots is false and the user is a bot, handle like immune
    if !config.target_bots.unwrap_or(true) && user_target.bot {
        return TakedownFail::Immune.into();
    }

    // same goes for the member
    let Some(member_target) = STATE.cache.member(guild_id, target_id)
    else {
        return TakedownFail::NotFound.into();
    };
    let member_target = member_target.deref();

    let is_random_and_silent_immune = random &&
        member_target
            .roles()
            .iter()
            .any(|role| config.immunity.random_silent.contains(role));
    if is_random_and_silent_immune {
        return TakedownFail::Immune.into();
    }

    let has_silent_immune_role = member_target
        .roles()
        .iter()
        .any(|role| config.immunity.general_silent.contains(role));
    if has_silent_immune_role {
        return TakedownFail::Immune.into();
    }

    // all preliminary checks are done here, we can start with joining the channel
    // and wululu

    let Some(channel_id) = STATE
        .cache
        .voice_state(target_id, guild_id)
        .map(|cvs| cvs.channel_id())
    else {
        return TakedownFail::NotFound.into();
    };

    // when we get this lock we can be sure no other takedown will take place on
    // this guild
    let _takedown_lock = STATE.takedown_locks.lock(guild_id);

    // that's why we can, with no doubt, insert the plan here
    let _planned_takedown_guard = STATE
        .planned_takedowns
        .upsert(guild_id, PlannedTakedown {
            _target_id: target_id,
            _channel_id: channel_id,
            _caller_id: caller_id.into(),
            countered_by_id: None
        })
        .expect("is unset as per guild this is a critical section");

    let call = match STATE.songbird.join(guild_id, channel_id).await {
        Ok(call) => call,
        Err(e) => return TakedownFail::Join(e).into()
    };
    let mut call = call.lock().await;

    let notification_sound = sounds::WULULULULU.deref();
    let takedown_sound = sounds::TAKEDOWNS
        .choose(rng)
        .expect("takedown sounds are not empty");
    let fail_sound = sounds::FAILS
        .choose(rng)
        .expect("fail sounds are not empty");

    let notification_track_handle =
        call.play_only_input(notification_sound.input.new_handle().into());

    let duration_multiplier = rng.gen_range(0.6..1.0);
    tokio::time::sleep(sounds::WULULULULU.duration.mul_f64(duration_multiplier)).await;

    // TODO: check if `stop` is ok, otherwise do `pause`
    notification_track_handle
        .stop()
        .expect("track is owned here");

    // while sleeping here, a counter could occur
    if let Some(countered_by_id) = STATE
        .planned_takedowns
        .read(guild_id, |plan| plan.countered_by_id)
        .expect("guard for that entry still alive")
    {
        let result = TakedownResult::Countered {
            by_id: countered_by_id
        };

        // do the counter measures here
        let Some(caller_id) = caller_id.into()
        else {
            // no caller_id, bot did it own it's own
            call.play_only_input(fail_sound.input.new_handle().into());
            tokio::time::sleep(fail_sound.duration + WAIT_AFTER_QUOTE).await;
            return result;
        };

        let caller_is_here = match STATE.cache.voice_state(caller_id, guild_id) {
            None => false,
            Some(caller_vs) => caller_vs.channel_id() == channel_id
        };

        let counter_is_immune = if let Some(counter) = STATE.cache.member(guild_id, countered_by_id)
        {
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

        // we do not need to check for bots as they can't invoke an application command

        return match caller_is_here && !counter_is_immune {
            true => {
                if let Err(err) = STATE
                    .http
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
                        return result;
                    }

                    return TakedownFail::HttpError(err).into();
                }

                takedown_sound
                    .play_and_wait(&mut call, WAIT_AFTER_QUOTE)
                    .await;
                result
            }
            false => {
                fail_sound.play_and_wait(&mut call, None).await;
                result
            }
        };
    }

    // TODO: check for fleeing

    // no counter, continue with takedown
    let has_immune_role = member_target
        .roles()
        .iter()
        .any(|role| config.immunity.general.contains(role));
    if has_immune_role {
        fail_sound.play_and_wait(&mut call, None).await;
        return TakedownFail::Immune.into();
    }

    if let Err(err) = STATE
        .http
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
            return TakedownFail::MissingPermissions.into();
        }

        return TakedownFail::HttpError(err).into();
    }

    takedown_sound
        .play_and_wait(&mut call, WAIT_AFTER_QUOTE)
        .await;
    TakedownResult::Success
}
