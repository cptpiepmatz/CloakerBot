use std::ops::Deref;
use std::sync::Arc;

use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::guild::Guild;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::{
    BooleanBuilder, ChannelBuilder, CommandBuilder, NumberBuilder, SubCommandBuilder
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::replies;
use crate::db::GuildConfig;
use crate::interactions::commands::TryFromInteractionError;
use crate::{Com, APPLICATION_ID};

pub enum ConfigCommand {
    Counter {
        enable: Option<bool>
    },
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
            SUB_COMMAND_COUNTER => {
                let mut enable: Option<bool> = None;
                let command = format!("{NAME} {SUB_COMMAND_COUNTER}");
                for option in options {
                    match option.name.as_str() {
                        "enable" => match &option.value {
                            CommandOptionValue::Boolean(b) => enable = Some(*b),
                            v => {
                                return Err(TryFromInteractionError::IncorrectFieldType {
                                    command,
                                    field: "enable".into(),
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
                Ok(ConfigCommand::Counter { enable })
            }
            SUB_COMMAND_DANGER_ZONES => {
                for option in options {
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
                            CommandOptionValue::Number(f) => min = Some(*f),
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
const SUB_COMMAND_COUNTER: &str = "counter";
const SUB_COMMAND_DANGER_ZONES: &str = "dangerzones";
const SUB_COMMAND_LOG_CHANNEL: &str = "logchannel";
const SUB_COMMAND_TARGET_BOTS: &str = "targetbots";
const SUB_COMMAND_RANDOM: &str = "random";

pub fn chat_input() -> Command {
    let log_channel = SubCommandBuilder::new(SUB_COMMAND_LOG_CHANNEL, "Define your log channel.")
        .option(ChannelBuilder::new("channel", "Channel to output logging data.").build())
        .build();
    let danger_zones =
        SubCommandBuilder::new(SUB_COMMAND_DANGER_ZONES, "Define your danger zones.").build();
    let counter = SubCommandBuilder::new(SUB_COMMAND_COUNTER, "Configure countering.")
        .option(BooleanBuilder::new("enable", "Enable/disable countering.").build())
        .build();
    let target_bots = SubCommandBuilder::new(
        SUB_COMMAND_TARGET_BOTS,
        "Configure if the bot should target bots."
    )
    .option(BooleanBuilder::new("allow", "Allow/disallow targeting bots.").build())
    .build();
    let random = SubCommandBuilder::new(SUB_COMMAND_RANDOM, "Configure random takedown behavior.")
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
        .build();

    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .dm_permission(false)
        .option(log_channel)
        .option(danger_zones)
        .option(counter)
        .option(target_bots)
        .option(random)
        .build()
}

pub async fn action(interaction: Interaction, com: Arc<Com>) {}
