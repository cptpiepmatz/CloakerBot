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
        .option(target_bots)
        .option(random)
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
        ConfigCommand::DangerZones => action_danger_zones(interaction, guild_id, com).await?,
        ConfigCommand::LogChannel { channel } => {
            action_log_channel(interaction, guild_id, channel, com).await?
        }
        ConfigCommand::Random { enable, min, max } => {
            action_random(interaction, guild_id, enable, min, max, com).await?
        }
        ConfigCommand::TargetBots { allow } => {
            action_target_bots(interaction, guild_id, allow, com).await?
        }
    }

    Ok(())
}

async fn action_danger_zones(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let config = guild_config(guild_id, com.clone()).await?;

    let content = match config.danger_zones.is_empty() {
        true => "**🛠️ *No* danger zones are currently set.**".to_owned(),
        false => format!(
            "**🛠️ The following channels are set as danger zones:** {}",
            config
                .danger_zones
                .iter()
                .map(|c| c.mention().to_string() + "\n")
                .collect::<String>()
        )
    };

    let payload = json!({
        "content": content,
        "components": [
            {
                "type": 1,
                "components": [{
                    // select menu for channel select
                    "type": 8,
                    "custom_id": format!("{}#{}", CONFIG_DANGER_ZONES_SELECT, interaction.id),
                    "channel_types": [ChannelType::GuildVoice],
                    "min_values": 0,
                    "max_values": 25,
                    "placeholder": "⚠️ Select channels as danger zones."
                }]
            },
            {
                "type": 1,
                "components": [{
                    // button to remove all channels
                    "type": 2,
                    "custom_id": format!("{}#{}", CONFIG_DANGER_ZONES_BUTTON, interaction.id),
                    "label": "Delete Danger Zones",
                    "style": ButtonStyle::Danger,
                    "emoji": {
                        "id": null,
                        "name": "🗑️"
                    }
                }]
            }
        ]
    });

    com.http
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .payload_json(payload.to_string().as_bytes())
        .await?;

    let interaction_id = interaction.id;
    com.tracked_interactions.insert(interaction.id, interaction);

    tokio::time::sleep(Duration::from_secs(3 * 60)).await;

    if let Some((_, interaction)) = com.tracked_interactions.remove(&interaction_id) {
        let payload = json!({
            "content": format!(
                "**⌛ Timed Out.**\nTry `/{} {}` again to modify danger zones.",
                NAME,
                SUB_COMMAND_DANGER_ZONES
            ),
            "components": []
        });

        com.http
            .interaction(APPLICATION_ID)
            .update_response(&interaction.token)
            .payload_json(payload.to_string().as_bytes())
            .await?;
    }

    Ok(())
}

async fn action_log_channel(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    log_channel_id: Option<Id<ChannelMarker>>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let mut config = guild_config(guild_id, com.clone()).await?;

    match (log_channel_id, config.log_channel) {
        (None, None) => {
            let log_channel_id = fetch_log_channel(guild_id, com.clone()).await?;
            let reply = match log_channel_id {
                Some(channel_id) => format!(
                    concat!(
                        "**🛠️ No log channel defined, will log into {}.**\n",
                        "Provide a log channel to select where to log.\n",
                        "With permissions you can deny the bot to send log into that channel."
                    ),
                    channel_id.mention()
                ),
                None => concat!(
                    "**🛠️ No log channel defined, no logging will occur.**\n",
                    "Provide a log channel to avoid this."
                )
                .into()
            };
            update_defer(&interaction, &reply, com.clone()).await?;
        }
        (None, Some(log_channel)) => {
            let channel_perms =
                fetch_in_channel_permissions(BOT_USER_ID, guild_id, log_channel, com.clone())
                    .await?;
            let reply = match channel_perms.contains(Permissions::SEND_MESSAGES) {
                true => format!("**🛠️ {} is set as log channel.**", log_channel.mention()),
                false => {
                    let fallback = fetch_log_channel(guild_id, com.clone()).await?;
                    match fallback {
                        Some(fallback_id) => format!(
                            concat!(
                                "**🛠️ {} is set as log channel, but will log into {}.**\n",
                                "The bot is not allowed to send messages into the log channel."
                            ),
                            log_channel.mention(),
                            fallback_id.mention()
                        ),
                        None => format!(
                            concat!(
                                "**🛠️ {} is set as log channel, but no logging will occur.**\n",
                                "The bot is not allowed to send messages into the log channel."
                            ),
                            log_channel.mention()
                        )
                    }
                }
            };
            update_defer(&interaction, &reply, com.clone()).await?;
        }
        (Some(log_channel), _) => {
            config.log_channel = Some(log_channel);
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let channel_perms =
                fetch_in_channel_permissions(BOT_USER_ID, guild_id, log_channel, com.clone())
                    .await?;
            let reply = match channel_perms.contains(Permissions::SEND_MESSAGES) {
                true => format!("**🛠️ Defined {} as log channel.**", log_channel.mention()),
                false => {
                    let fallback_channel_id = fetch_log_channel(guild_id, com.clone()).await?;
                    match fallback_channel_id {
                        Some(fallback) => format!(
                            concat!(
                                "**🛠️ Defined {} as log channel, but will log into {}.**\n",
                                "The bot is not allowed to send messages into the log channel."
                            ),
                            log_channel.mention(),
                            fallback.mention()
                        ),
                        None => format!(
                            concat!(
                                "**🛠️ Defined {} as log channel, no logging will occur.**\n",
                                "The bot is not allowed to send messages into the log channel."
                            ),
                            log_channel.mention()
                        )
                    }
                }
            };
            update_defer(&interaction, &reply, com.clone()).await?;
        }
    }

    Ok(())
}

async fn action_random(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    enable: Option<bool>,
    min: Option<f64>,
    max: Option<f64>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let mut config = guild_config(guild_id, com.clone()).await?;

    match (enable, min, max) {
        (Some(false), _, _) => {
            config.random.enabled = false;
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let reply = "**🛠️ *Disabled* random takedowns on this server.**";
            update_defer(&interaction, reply, com.clone()).await?;
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
            update_defer(&interaction, &reply, com.clone()).await?;
        }
        (_, min, max) => {
            let min = min.unwrap_or(config.random.min);
            let max = max.unwrap_or(config.random.max);
            config.random = GuildConfigRandom {
                enabled: true,
                min,
                max
            };
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let reply = format!(
                concat!(
                    "**🛠️ ",
                    "*Enabled* random takedowns on this server.**\n",
                    "The time interval for random takedowns is set to {}-{}."
                ),
                min, max
            );
            update_defer(&interaction, &reply, com.clone()).await?;
        }
    }

    Ok(())
}

async fn action_target_bots(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    allow: Option<bool>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let mut config = guild_config(guild_id, com.clone()).await?;

    match allow {
        None => {
            let reply = match config.target_bots {
                true => "**🛠️ Taking down bots is *allowed* on this server.**",
                false => "**🛠️ Taking down bots is *disallowed* on this server.**"
            };
            update_defer(&interaction, reply, com.clone()).await?;
        }
        Some(allow) => {
            config.target_bots = allow;
            let _: Option<GuildConfig> = com
                .db
                .update((db::config::TABLE_NAME, guild_id.get()))
                .content(config)
                .await?;
            let reply = match allow {
                true => "**🛠️ *Allowed* taking down bots on this server.**",
                false => "**🛠️ *Disallowed* taking down bots on this server.**"
            };
            update_defer(&interaction, reply, com.clone()).await?;
        }
    }

    Ok(())
}

async fn guild_config(guild_id: Id<GuildMarker>, com: Arc<Com>) -> anyhow::Result<GuildConfig> {
    let db_key = (db::config::TABLE_NAME, guild_id.get());
    let config_db: Option<GuildConfig> = com.db.select(db_key).await?;
    Ok(config_db.unwrap_or_default())
}
