use twilight_mention::Mention;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::{
    CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::guild::Permissions;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;
use twilight_util::builder::command::{ChannelBuilder, SubCommandBuilder};

use crate::db::config::GuildConfig;
use crate::interaction::application_command::TryFromInteractionError;
use crate::BOT_USER_ID;

pub const NAME: &str = "log-channel";
const DESCRIPTION: &str = "Define your log channel.";

pub struct LogChannelSubCommand {
    channel_id: Option<Id<ChannelMarker>>
}

impl TryFrom<&Vec<CommandDataOption>> for LogChannelSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        let mut channel: Option<Id<ChannelMarker>> = None;
        let command = format!("{} {}", super::NAME, NAME);
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
        Ok(LogChannelSubCommand {
            channel_id: channel
        })
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION)
        .option(ChannelBuilder::new("channel", "Channel to output logging data.").build())
        .build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    LogChannelSubCommand { channel_id }: LogChannelSubCommand
) -> anyhow::Result<()> {
    let mut config = GuildConfig::fetch(guild_id).await?;

    match (channel_id, config.log_channel) {
        (None, None) => {
            let log_channel_id = crate::log::fetch_log_channel(guild_id).await?;
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
            super::super::update_defer(&interaction, &reply).await?;
        }
        (None, Some(log_channel)) => {
            let channel_perms = crate::permissions::fetch_in_channel_permissions(
                BOT_USER_ID,
                guild_id,
                log_channel
            )
            .await?;
            let reply = match channel_perms.contains(Permissions::SEND_MESSAGES) {
                true => format!("**🛠️ {} is set as log channel.**", log_channel.mention()),
                false => {
                    let fallback = crate::log::fetch_log_channel(guild_id).await?;
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
            super::super::update_defer(&interaction, &reply).await?;
        }
        (Some(log_channel), _) => {
            config.log_channel = Some(log_channel);
            config.save(guild_id).await?;
            let channel_perms = crate::permissions::fetch_in_channel_permissions(
                BOT_USER_ID,
                guild_id,
                log_channel
            )
            .await?;
            let reply = match channel_perms.contains(Permissions::SEND_MESSAGES) {
                true => format!("**🛠️ Defined {} as log channel.**", log_channel.mention()),
                false => {
                    let fallback_channel_id = crate::log::fetch_log_channel(guild_id).await?;
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
            super::super::update_defer(&interaction, &reply).await?;
        }
    }

    Ok(())
}
