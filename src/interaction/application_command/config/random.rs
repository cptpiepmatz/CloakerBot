use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{CommandMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, NumberBuilder, SubCommandBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::ApplicationCommand::Config;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::sync::OnceLockExpect;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "random";
pub const DESCRIPTION: &str = "Configure random takedown behavior.";

pub const CLOAKER_QUOTE_TARGET_BOTS: &str =
    "They just gave a tiger a free pass in a playground. Better run, kids!";
pub const CLOAKER_QUOTE_NO_TARGET_BOTS: &str =
    "No more free-for-all fireworks. The party-poopers have spoken.";

pub struct RandomSubCommand {
    enable: Option<bool>,
    min: Option<f64>,
    max: Option<f64>
}

impl TryFrom<&Vec<CommandDataOption>> for RandomSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut enable: Option<bool> = None;
        let mut min: Option<f64> = None;
        let mut max: Option<f64> = None;
        let command = format!("{} {}", super::NAME, NAME);
        for option in options {
            match option.name.as_str() {
                "enable" => match &option.value {
                    CommandOptionValue::Boolean(b) => enable = Some(*b),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "allow".into(),
                            value: v.clone()
                        })
                    }
                },
                "min" => match &option.value {
                    CommandOptionValue::Number(f) => min = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "min".into(),
                            value: v.clone()
                        })
                    }
                },
                "max" => match &option.value {
                    CommandOptionValue::Number(f) => max = Some(*f),
                    v => {
                        return Err(TryFromInteractionError::IncorrectFieldType {
                            command,
                            field: "max".into(),
                            value: v.clone()
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
        Ok(RandomSubCommand { enable, min, max })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(BooleanBuilder::new("enable", "Enable/disable random takedowns.").build())
        .option(
            NumberBuilder::new(
                "min",
                "Minimum time between takedowns. `-1` to set default value."
            )
            .build()
        )
        .option(
            NumberBuilder::new(
                "max",
                "Maximum time between takedowns. `-1` to set default value."
            )
            .build()
        )
        .build()
}

pub async fn action(
    interaction: &Interaction,
    RandomSubCommand { enable, min, max }: RandomSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();

    let current = &guild_config.random.clone();

    if let Some(enable) = enable {
        guild_config.random.enabled = Some(enable);
    }

    if let Some(min) = min {
        guild_config.random.min = Some(min);
    }

    if let Some(max) = max {
        guild_config.random.max = Some(max);
    }

    let content = "updated";

    let response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .build();

    STATE
        .http
        .interaction(APPLICATION_ID)
        .create_response(
            interaction.id,
            interaction.token.as_str(),
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(response_data)
            }
        )
        .await?;

    Ok(())
}
