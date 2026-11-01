use std::sync::LazyLock;

use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_util::builder::command::{NumberBuilder, SubCommandBuilder};

use crate::STATE;
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::modal_submit;

pub const NAME: &str = "random";
pub const DESCRIPTION: &str = "Configure random takedown behavior.";
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::super::mention![subcommand];

pub struct RandomSubCommand {
    min: Option<f64>,
    max: Option<f64>
}

impl TryFrom<&Vec<CommandDataOption>> for RandomSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "min" => match &option.value {
                    CommandOptionValue::Number(f) => min = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "min".into(),
                            value: v.clone()
                        });
                    }
                },
                "max" => match &option.value {
                    CommandOptionValue::Number(f) => max = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "max".into(),
                            value: v.clone()
                        });
                    }
                },
                n => return Err(TryFromInteractionError::UnknownField { command, field: n.into() })
            }
        }
        Ok(RandomSubCommand { min, max })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(
            NumberBuilder::new(
                "min",
                "Prefill minimum interval in hours. Submit the form to save. -1 restores default."
            )
            .build()
        )
        .option(
            NumberBuilder::new(
                "max",
                "Prefill maximum interval in hours. Submit the form to save. -1 restores default."
            )
            .build()
        )
        .build()
}

pub async fn action(
    interaction: &Interaction,
    RandomSubCommand { min, max }: RandomSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let mut random = STATE.database.guild_config(guild_id).await?.random;
    if min.is_some() || max.is_some() {
        random.enabled = Some(true);
    }
    if let Some(min) = min {
        random.min = (min >= 0.0).then_some(min);
    }
    if let Some(max) = max {
        random.max = (max >= 0.0).then_some(max);
    }
    let response_data = modal_submit::config::random::modal(&random);

    super::respond_with_modal(interaction, response_data).await
}
