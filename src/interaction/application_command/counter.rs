use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;

use thiserror::Error;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_mention::Mention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::CommandMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::CommandBuilder;

use super::COMMANDS_REGISTER;
use crate::assets::replies;
use crate::interaction::application_command;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::takedown;

pub const NAME: &str = "counter";
pub const DESCRIPTION: &str = "Counter an incoming takedown.";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.counter);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

pub struct CounterCommand;

impl TryFrom<&CommandData> for CounterCommand {
    type Error = TryFromInteractionError;

    fn try_from(_value: &CommandData) -> Result<Self, Self::Error> {
        Ok(CounterCommand)
    }
}

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput).dm_permission(false).build()
}

pub fn user_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::User).dm_permission(false).build()
}

pub fn message_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::Message).dm_permission(false).build()
}

#[derive(Debug, Error)]
pub enum CounterActionError {
    #[error("not in guild")]
    NotInGuild,
    #[error("no author found")]
    NoAuthor
}

pub async fn action(
    interaction: &Interaction,
    _command: CounterCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(CounterActionError::NotInGuild)?;
    let author_id = interaction.author_id().ok_or(CounterActionError::NoAuthor)?;
    application_command::defer(interaction).await?;

    let content = match takedown::counter::counter(guild_id, author_id) {
        Ok(target_id) => replies::counter::success(
            // TODO: add quote here
            "[insert quote here]",
            target_id.mention()
        ),
        Err(()) => replies::counter::miss(
            // TODO: add another quote here
            "[insert quote here]"
        )
    };

    application_command::update_defer_msg(interaction, &content).await?;

    Ok(())
}
