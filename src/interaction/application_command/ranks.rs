use std::sync::LazyLock;

use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_mention::Mention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::{CommandMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, UserBuilder};

use super::COMMANDS_REGISTER;
use crate::data::stats;
use crate::data::stats::GuildStats;
use crate::interaction::application_command;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::{takedown, STATE};

pub const NAME: &str = "ranks";
pub const DESCRIPTION: &str = "Display your takedown stats in a rank badge.";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.ranks);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

#[derive(Debug)]
pub struct RanksCommand {
    user_id: Option<Id<UserMarker>>
}

impl TryFrom<&CommandData> for RanksCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let mut user: Option<Id<UserMarker>> = None;
        for option in &data.options {
            let CommandDataOption { name, value } = option;
            match (name.as_str(), value) {
                ("user", CommandOptionValue::User(u)) => user = Some(*u),
                ("user", _) => {
                    return Err(TryFromInteractionError::IncorrectFieldType {
                        command: NAME.into(),
                        field: name.clone(),
                        value: value.clone()
                    })
                }
                (_, _) => {
                    return Err(TryFromInteractionError::UnknownField {
                        command: NAME.into(),
                        field: name.clone()
                    })
                }
            }
        }

        Ok(RanksCommand { user_id: user })
    }
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("user", "User to show ranks for.").required(false).build();

    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .option(option)
        .build()
}

#[derive(Debug)]
pub enum RanksActionError {
    NotInGuild,
    NoAuthor
}

pub async fn action(
    interaction: &Interaction,
    command: RanksCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(RanksActionError::NotInGuild)?;
    let author_id = interaction.author_id().ok_or(RanksActionError::NoAuthor)?;
    application_command::defer(interaction).await?;

    let user_id = command.user_id.unwrap_or(author_id);

    let guild_stats = STATE.guild_stats.read().await;
    let default = GuildStats::default();
    let this_guild_stats = guild_stats.get(&guild_id).unwrap_or(&default);
    let global_takedown_count = stats::global_takedown_count(guild_stats.values());
    let mut global_takedown_count_ranked: Vec<(Id<UserMarker>, u64)> =
        global_takedown_count.iter().map(|(&caller_id, &count)| (caller_id, count)).collect();
    global_takedown_count_ranked.sort_by(|(_, a), (_, b)| b.cmp(a));

    let local_takedown_count: u64 = this_guild_stats.takedowns.count(user_id);
    let global_takedown_count: u64 =
        global_takedown_count.get(&user_id).copied().unwrap_or_default();

    let local_rank: Option<usize> = this_guild_stats
        .takedowns
        .ranked()
        .enumerate()
        .map(|(pos, (caller_id, _))| (pos + 1, caller_id))
        .find(|(_, caller_id)| *caller_id == user_id)
        .map(|(pos, _)| pos);
    let global_rank: Option<usize> = global_takedown_count_ranked
        .iter()
        .enumerate()
        .map(|(pos, (caller_id, _))| (pos + 1, caller_id))
        .find(|(_, caller_id)| **caller_id == user_id)
        .map(|(pos, _)| pos);

    // let content = match takedown::counter::counter(guild_id, author_id) {
    //     Ok(target_id) =>
    // REPLIES.counter.success.template([&target_id.mention()]),     Err(()) =>
    // REPLIES.counter.miss.template([]) };
    //
    // application_command::update_defer_msg(interaction, &content).await?;
    //
    // Ok(())

    todo!()
}
