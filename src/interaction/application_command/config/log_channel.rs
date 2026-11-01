use std::sync::LazyLock;

use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_util::builder::command::SubCommandBuilder;

use crate::STATE;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::modal_submit;

pub const NAME: &str = "log-channel";
pub const DESCRIPTION: &str = "Define your log channel.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

pub const CLOAKER_QUOTE_LOG_CHANNEL: &str =
    "I'd rather eat glass than report in, but hey, let’s not disappoint the bureaucracy.";

pub struct LogChannelSubCommand;

impl TryFrom<&Vec<CommandDataOption>> for LogChannelSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        if let Some(option) = options.iter().next() {
            return Err(TryFromInteractionError::UnknownField {
                command: format!("{} {}", super::NAME, NAME),
                field: String::from(&option.name)
            });
        }
        Ok(LogChannelSubCommand)
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION).build()
}

pub async fn action(
    interaction: &Interaction,
    _command: LogChannelSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let current = STATE.database.guild_config(guild_id).await?.log_channel;

    let response_data = modal_submit::config::log_channel::modal(current);

    super::respond_with_modal(interaction, response_data).await
}
