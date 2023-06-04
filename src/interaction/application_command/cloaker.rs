use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::guild::Permissions;
use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{CommandBuilder, UserBuilder};

use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};

#[derive(Debug)]
pub struct CloakerCommand {
    target_id: Id<UserMarker>
}

impl TryFrom<&CommandData> for CloakerCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let mut target: Option<Id<UserMarker>> = None;
        for option in &data.options {
            let CommandDataOption { name, value } = option;
            match name.as_str() {
                "target" => match value {
                    CommandOptionValue::User(u) => target = Some(*u),
                    _v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command: NAME.into(),
                            field: name.clone(),
                            value: value.clone()
                        })
                    }
                },
                _n => {
                    return Err(TryFromInteractionError::UnknownField {
                        command: NAME.into(),
                        field: name.clone()
                    })
                }
            }
        }

        match target {
            None => Err(TryFromInteractionError::MissingField {
                command: NAME.into(),
                field: "target".into()
            }),
            Some(target) => Ok(CloakerCommand { target_id: target })
        }
    }
}

pub const NAME: &str = "cloaker";
const DESCRIPTION: &str = "Take down your friends!";

fn permissions() -> Permissions {
    Permissions::ADMINISTRATOR | Permissions::MODERATE_MEMBERS | Permissions::MOVE_MEMBERS
}

pub fn chat_input() -> Command {
    let option = UserBuilder::new("target", "User to take down.")
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

#[derive(Debug)]
pub enum CloakerActionError {
    NotInGuild,
    NoUser
}

impl Display for CloakerActionError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for CloakerActionError {}

impl From<CloakerActionError> for HandleApplicationCommandError {
    fn from(value: CloakerActionError) -> Self {
        HandleApplicationCommandError::CloakerAction(value)
    }
}

pub async fn action(
    interaction: &Interaction,
    command: CloakerCommand
) -> Result<(), HandleApplicationCommandError> {
    todo!()
}
