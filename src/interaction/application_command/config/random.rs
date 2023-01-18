use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{BooleanBuilder, NumberBuilder, SubCommandBuilder};

use crate::db::config::{GuildConfig, GuildConfigRandom};
use crate::db::DataStoreEntry;
use crate::interaction::application_command::TryFromInteractionError;

pub const NAME: &str = "random";
const DESCRIPTION: &str = "Configure random takedown behavior.";

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
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    RandomSubCommand { enable, min, max }: RandomSubCommand
) -> anyhow::Result<()> {
    let mut config = GuildConfig::fetch(guild_id).await?;

    match (enable, min, max) {
        (Some(false), _, _) => {
            config.random.enabled = false;
            config.store(guild_id).await?;
            let reply = "**🛠️ *Disabled* random takedowns on this server.**";
            super::super::update_defer(&interaction, reply).await?;
        }
        (None, None, None) => {
            let reply = match config.random.enabled {
                false => String::from("**🛠️ Random takedowns are *disabled* on this server.**"),
                true => format!(
                    concat!(
                        "**🛠️ ",
                        "Random takedowns are *enabled* on this server.**\n",
                        "The time interval for random takedowns is set to {}-{}."
                    ),
                    config.random.min, config.random.max
                )
            };
            super::super::update_defer(&interaction, &reply).await?;
        }
        (_, min, max) => {
            let min = min.unwrap_or(config.random.min);
            let max = max.unwrap_or(config.random.max);
            config.random = GuildConfigRandom {
                enabled: true,
                min,
                max
            };
            config.store(guild_id).await?;
            let reply = format!(
                concat!(
                    "**🛠️ ",
                    "*Enabled* random takedowns on this server.**\n",
                    "The time interval for random takedowns is set to {}-{}."
                ),
                min, max
            );
            super::super::update_defer(&interaction, &reply).await?;
        }
    }

    Ok(())
}
