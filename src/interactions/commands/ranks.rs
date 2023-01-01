use twilight_model::application::command::{Command, CommandType};
use twilight_util::builder::command::CommandBuilder;

pub const NAME: &str = "ranks";
const DESCRIPTION: &str = "Display your takedown stats in a rank badge.";

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .build()
}
