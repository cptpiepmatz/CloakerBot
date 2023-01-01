use twilight_model::application::command::{Command, CommandType};
use twilight_model::guild::Permissions;
use twilight_util::builder::command::{CommandBuilder, UserBuilder};

const NAME: &str = "cloaker";
const DESCRIPTION: &str = "Take down your friends!";

fn permissions() -> Permissions {
    Permissions::ADMINISTRATOR | Permissions::MODERATE_MEMBERS | Permissions::MOVE_MEMBERS
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("user", "User to take down.")
        .required(true)
        .build();
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .default_member_permissions(permissions())
        .dm_permission(false)
        .option(option)
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
