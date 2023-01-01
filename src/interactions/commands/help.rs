use twilight_model::application::command::{Command, CommandType};
use twilight_util::builder::command::CommandBuilder;

pub const NAME: &str = "help";
const DESCRIPTION: &str = "Shows a help page.";

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .build()
}
