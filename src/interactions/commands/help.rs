use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_util::builder::command::CommandBuilder;

use crate::interactions::commands::TryFromInteractionError;

pub struct HelpCommand;

impl TryFrom<&CommandData> for HelpCommand {
    type Error = TryFromInteractionError;

    fn try_from(_value: &CommandData) -> Result<Self, Self::Error> {
        Ok(HelpCommand)
    }
}

pub const NAME: &str = "help";
const DESCRIPTION: &str = "Shows a help page.";

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .build()
}
