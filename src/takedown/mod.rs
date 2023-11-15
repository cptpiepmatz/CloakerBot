use std::ops::Deref;

use derive_more::From;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::data::config::GuildConfig;
use crate::{BOT_USER_ID, STATE};

pub mod sounds;

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

#[derive(Debug)]
pub enum TakedownFail {
    // TODO: maybe this needs a variant for other discord-related errors

    // TODO: insert the error type for missing perms here
    /// Missing permissions to either join the channel or kick the target.
    MissingPermissions(),
    /// Could not find the user.
    ///
    /// The channel could be not visible to the bot or the user is currently not
    /// a channel on that guild.
    NotFound,
    /// The target fled before the bot could finish its takedown.
    Fled,
    /// The target is immune.
    Immune
}

/// Takedown plan.
///
/// Applied to a hash map with guilds as keys.
#[derive(Debug)]
pub struct PlannedTakedown {
    target_id: Id<UserMarker>,
    channel_id: Id<ChannelMarker>,
    /// Caller of the takedown plan.
    ///
    /// If [`None`], the bot itself invoked this.
    caller: Option<Id<UserMarker>>
}

pub async fn takedown(
    target_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    caller_id: impl Into<Option<Id<UserMarker>>>,
    random: bool
) -> TakedownResult {
    // get the caller id, if no called is used here, the bot initiated own its own
    // (e.g. danger zones)
    let caller_id = caller_id.into().unwrap_or(BOT_USER_ID);

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
    let target = member_target.deref();

    let is_random_and_silent_immune = random &&
        target
            .roles()
            .iter()
            .any(|role| config.immunity.random_silent.contains(role));
    if is_random_and_silent_immune {
        return TakedownFail::Immune.into();
    }

    let has_silent_immune_role = target
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

    todo!()
}
