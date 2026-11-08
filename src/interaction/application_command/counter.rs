use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;

use thiserror::Error;
use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::application::interaction::{Interaction, InteractionContextType};
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::command::CommandBuilder;

use super::COMMANDS_REGISTER;
use crate::assets::templates;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::{application_command, defer};
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
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .contexts([InteractionContextType::Guild])
        .build()
}

pub fn user_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::User)
        .contexts([InteractionContextType::Guild])
        .build()
}

pub fn message_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::Message)
        .contexts([InteractionContextType::Guild])
        .build()
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
    let result = takedown::counter::counter(guild_id, author_id);
    defer::ephemeral(interaction).await?;

    let content = match result {
        Ok(target_id) => {
            templates::counter::success(target_id.mention(), crate::assets::QUOTES.counter.success)
                .to_string()
        }
        Err(()) => templates::counter::miss(crate::assets::QUOTES.counter.miss).to_string()
    };

    application_command::update_defer_msg(interaction, &content).await?;

    Ok(())
}
