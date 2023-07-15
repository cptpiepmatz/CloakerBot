use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::command::CommandBuilder;

use crate::interaction::application_command::TryFromInteractionError;
use crate::sync::OnceLockExpect;

pub const NAME: &str = "counter";
pub const DESCRIPTION: &str = "Counter an incoming takedown.";
pub static ID: OnceLockExpect<Id<CommandMarker>> = OnceLockExpect::new();

pub struct CounterCommand;

impl TryFrom<&CommandData> for CounterCommand {
    type Error = TryFromInteractionError;

    fn try_from(_value: &CommandData) -> Result<Self, Self::Error> {
        Ok(CounterCommand)
    }
}

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .build()
}

pub fn user_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::User)
        .dm_permission(false)
        .build()
}

pub fn message_context_menu() -> Command {
    CommandBuilder::new(NAME, "", CommandType::Message)
        .dm_permission(false)
        .build()
}
