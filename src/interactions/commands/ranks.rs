use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_util::builder::command::CommandBuilder;

use crate::interactions::commands::TryFromInteractionError;

#[derive(Debug)]
pub struct RanksCommand;

impl TryFrom<&CommandData> for RanksCommand {
    type Error = TryFromInteractionError;

    fn try_from(_value: &CommandData) -> Result<Self, Self::Error> {
        Ok(RanksCommand)
    }
}

pub const NAME: &str = "ranks";
const DESCRIPTION: &str = "Display your takedown stats in a rank badge.";

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .build()
}
