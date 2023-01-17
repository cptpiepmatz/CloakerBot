pub mod danger_zones;
pub mod log_channel;
pub mod random;
pub mod target_bots;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::CommandBuilder;

use crate::db::config::GuildConfig;
use crate::interaction::application_command::{defer, TryFromInteractionError};
use crate::{db, Com};

pub const NAME: &str = "config";
const DESCRIPTION: &str = "Configure the CloakerBot.";

pub enum ConfigCommand {
    DangerZones(danger_zones::DangerZonesSubCommand),
    LogChannel(log_channel::LogChannelSubCommand),
    Random(random::RandomSubCommand),
    TargetBots(target_bots::TargetBotsSubCommand)
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
            danger_zones::NAME => Ok(ConfigCommand::DangerZones(
                danger_zones::DangerZonesSubCommand::try_from(options)?
            )),
            log_channel::NAME => Ok(ConfigCommand::LogChannel(
                log_channel::LogChannelSubCommand::try_from(options)?
            )),
            target_bots::NAME => Ok(ConfigCommand::TargetBots(
                target_bots::TargetBotsSubCommand::try_from(options)?
            )),
            random::NAME => Ok(ConfigCommand::Random(random::RandomSubCommand::try_from(
                options
            )?)),
            n => Err(TryFromInteractionError::UnknownCommandName {
                name: format!("{NAME} {}", n)
            })
        }
    }
}

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
        ConfigCommand::DangerZones(sub_cmd) => {
            danger_zones::action(interaction, guild_id, sub_cmd, com).await?
        }
        ConfigCommand::LogChannel(sub_cmd) => {
            log_channel::action(interaction, guild_id, sub_cmd, com).await?
        }
        ConfigCommand::TargetBots(sub_cmd) => {
            target_bots::action(interaction, guild_id, sub_cmd, com).await?
        }
        ConfigCommand::Random(sub_cmd) => {
            random::action(interaction, guild_id, sub_cmd, com).await?
        }
    }

    Ok(())
}

async fn guild_config(guild_id: Id<GuildMarker>, com: Arc<Com>) -> anyhow::Result<GuildConfig> {
    let db_key = (db::config::TABLE_NAME, guild_id.get());
    let config_db: Option<GuildConfig> = com.db.select(db_key).await?;
    Ok(config_db.unwrap_or_default())
}

async fn update_db(
    config: GuildConfig,
    guild_id: Id<GuildMarker>,
    com: Arc<Com>
) -> anyhow::Result<()> {
    let _: Option<GuildConfig> = com
        .db
        .update((db::config::TABLE_NAME, guild_id.get()))
        .content(config)
        .await?;

    Ok(())
}
