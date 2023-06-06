use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{StringBuilder, SubCommandBuilder};

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::TryFromInteractionError;

pub const NAME: &str = "immunity";
pub const DESCRIPTION: &str = "Define what roles are immune to takedowns.";

pub enum ImmunitySubCommand {
    General,
    Random
}

impl TryFrom<&Vec<CommandDataOption>> for ImmunitySubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut for_general = false;
        let mut for_random = false;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "type" => match &option.value {
                    CommandOptionValue::String(s) => match s.as_str() {
                        "general" => for_general = true,
                        "random" => for_random = true,
                        _ => {
                            return Err(TryFromInteractionError::IncorrectFieldValue {
                                command,
                                field: "type".into(),
                                value: option.value.clone()
                            })
                        }
                    },
                    _n => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "type".into(),
                            value: option.value.clone()
                        })
                    }
                },
                n => {
                    return Err(TryFromInteractionError::UnknownField {
                        command,
                        field: n.into()
                    })
                }
            }
        }

        match (for_general, for_random) {
            (true, _) => Ok(ImmunitySubCommand::General),
            (_, true) => Ok(ImmunitySubCommand::Random),
            (false, false) => Err(TryFromInteractionError::MissingField {
                command,
                field: "type".into()
            })
        }
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            StringBuilder::new("type", "Select which type of immunity role.")
                .choices([
                    ("general".to_owned(), "general".to_owned()),
                    ("random".to_owned(), "random".to_owned())
                ])
                .required(true)
        )
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    command: ImmunitySubCommand
) -> Result<(), ConfigActionError> {
    todo!()
}
