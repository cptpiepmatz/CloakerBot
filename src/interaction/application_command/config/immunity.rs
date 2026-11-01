use std::sync::LazyLock;

use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_util::builder::command::{StringBuilder, SubCommandBuilder};

use crate::STATE;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::modal_submit;
use crate::interaction::modal_submit::config::immunity::ImmunityModal;

pub const NAME: &str = "immunity";
pub const DESCRIPTION: &str = "Define what roles are immune to takedowns.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

pub enum ImmunitySubCommand {
    General,
    Random,
    GeneralSilent,
    RandomSilent
}

impl TryFrom<&Vec<CommandDataOption>> for ImmunitySubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut for_general = false;
        let mut for_random = false;
        let mut for_general_silent = false;
        let mut for_random_silent = false;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "type" => match &option.value {
                    CommandOptionValue::String(s) => match s.as_str() {
                        "general" => for_general = true,
                        "random" => for_random = true,
                        "general-silent" => for_general_silent = true,
                        "random-silent" => for_random_silent = true,
                        _ => {
                            return Err(TryFromInteractionError::IncorrectFieldValue {
                                command,
                                field: "type".into(),
                                value: option.value.clone()
                            });
                        }
                    },
                    _n => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "type".into(),
                            value: option.value.clone()
                        });
                    }
                },
                n => return Err(TryFromInteractionError::UnknownField { command, field: n.into() })
            }
        }

        match (for_general, for_random, for_general_silent, for_random_silent) {
            (true, _, _, _) => Ok(ImmunitySubCommand::General),
            (_, true, _, _) => Ok(ImmunitySubCommand::Random),
            (_, _, true, _) => Ok(ImmunitySubCommand::GeneralSilent),
            (_, _, _, true) => Ok(ImmunitySubCommand::RandomSilent),
            (false, false, false, false) => {
                Err(TryFromInteractionError::MissingField { command, field: "type".into() })
            }
        }
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            StringBuilder::new("type", "Select which type of immunity role.")
                .choices([
                    ("general".to_owned(), "general".to_owned()),
                    ("random".to_owned(), "random".to_owned()),
                    ("general-silent".to_owned(), "general-silent".to_owned()),
                    ("random-silent".to_owned(), "random-silent".to_owned())
                ])
                .required(true)
        )
        .build()
}

pub async fn action(
    interaction: &Interaction,
    command: ImmunitySubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let config = STATE.database.guild_config(guild_id).await?;
    let variant = match command {
        ImmunitySubCommand::General => ImmunityModal::General,
        ImmunitySubCommand::Random => ImmunityModal::Random,
        ImmunitySubCommand::GeneralSilent => ImmunityModal::GeneralSilent,
        ImmunitySubCommand::RandomSilent => ImmunityModal::RandomSilent
    };
    let response_data = modal_submit::config::immunity::modal(variant, &config.immunity);

    super::respond_with_modal(interaction, response_data).await
}
