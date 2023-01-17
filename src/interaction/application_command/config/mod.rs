pub mod danger_zones;
pub mod log_channel;
pub mod random;
pub mod target_bots;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use twilight_mention::Mention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::channel::ChannelType;
use twilight_model::guild::Permissions;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{
    BooleanBuilder, ChannelBuilder, CommandBuilder, NumberBuilder, SubCommandBuilder
};

use crate::db::config::{GuildConfig, GuildConfigRandom};
use crate::interaction::application_command::{defer, update_defer, TryFromInteractionError};
use crate::interaction::message_component::custom_id::{
    CONFIG_DANGER_ZONES_BUTTON, CONFIG_DANGER_ZONES_SELECT
};
use crate::log::fetch_log_channel;
use crate::permissions::fetch_in_channel_permissions;
use crate::{db, Com, APPLICATION_ID, BOT_USER_ID};

pub enum ConfigCommand {
    DangerZones,
    LogChannel {
        channel: Option<Id<ChannelMarker>>
    },
    Random {
        enable: Option<bool>,
        min: Option<f64>,
        max: Option<f64>
    },
    TargetBots {
        allow: Option<bool>
    }
}

impl TryFrom<&CommandData> for ConfigCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let Some(CommandDataOption {name, value}) = data.options.first() else {
            return Err(TryFromInteractionError::MissingField {command: NAME.into(), field: "subcommand".into()})
        };

        let CommandOptionValue::SubCommand(options) = value else {
            return Err(TryFromInteractionError::IncorrectFieldType {command: NAME.into(), field: "subcommand".into(), value: value.clone() })
        };

        match name.as_str() {
            SUB_COMMAND_DANGER_ZONES => {
                if let Some(option) = options.iter().next() {
                    return Err(TryFromInteractionError::UnknownField {
                        command: format!("{NAME} {SUB_COMMAND_DANGER_ZONES}"),
                        field: String::from(&option.name)
                    });
                }
                Ok(ConfigCommand::DangerZones)
            }
            SUB_COMMAND_LOG_CHANNEL => {
                let mut channel: Option<Id<ChannelMarker>> = None;
                let command = format!("{NAME} {SUB_COMMAND_LOG_CHANNEL}");
                for option in options {
                    match option.name.as_str() {
                        "channel" => match &option.value {
                            CommandOptionValue::Channel(c) => channel = Some(*c),
                            v => {
                                return Err(TryFromInteractionError::IncorrectFieldType {
                                    command,
                                    field: "channel".into(),
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
                Ok(ConfigCommand::LogChannel { channel })
            }
            SUB_COMMAND_TARGET_BOTS => {
                let mut allow: Option<bool> = None;
                let command = format!("{NAME} {SUB_COMMAND_TARGET_BOTS}");
                for option in options {
                    match option.name.as_str() {
                        "allow" => match &option.value {
                            CommandOptionValue::Boolean(b) => allow = Some(*b),
                            v => {
                                return Err(TryFromInteractionError::IncorrectFieldType {
                                    command,
                                    field: "allow".into(),
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
                Ok(ConfigCommand::TargetBots { allow })
            }
            SUB_COMMAND_RANDOM => {
                let mut enable: Option<bool> = None;
                let mut min: Option<f64> = None;
                let mut max: Option<f64> = None;
                let command = format!("{NAME} {SUB_COMMAND_RANDOM}");
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
                Ok(ConfigCommand::Random { enable, min, max })
            }
            n => Err(TryFromInteractionError::UnknownCommandName {
                name: format!("{NAME} {}", n)
            })
        }
    }
}

pub const NAME: &str = "config";
const DESCRIPTION: &str = "Configure the CloakerBot.";

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .option(log_channel::chat_input())
        .option(danger_zones::chat_input())
        .option(target_bots::chat_input())
        .option(random::chat_input())
        .build()
}

#[derive(Debug)]
pub enum ConfigActionError {
    NotInGuild
}

impl Display for ConfigActionError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for ConfigActionError {}

pub async fn action(
    interaction: Interaction,
    command: ConfigCommand,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let guild_id = interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    defer(&interaction, com.clone()).await?;

    match command {
        ConfigCommand::DangerZones => danger_zones::action(interaction, guild_id, com).await?,
        ConfigCommand::LogChannel { channel } => {
            log_channel::action(interaction, guild_id, channel, com).await?
        }
        ConfigCommand::Random { enable, min, max } => {
            random::action(interaction, guild_id, enable, min, max, com).await?
        }
        ConfigCommand::TargetBots { allow } => {
            target_bots::action(interaction, guild_id, allow, com).await?
        }
    }

    Ok(())
}

async fn guild_config(guild_id: Id<GuildMarker>, com: Arc<Com>) -> anyhow::Result<GuildConfig> {
    let db_key = (db::config::TABLE_NAME, guild_id.get());
    let config_db: Option<GuildConfig> = com.db.select(db_key).await?;
    Ok(config_db.unwrap_or_default())
}
