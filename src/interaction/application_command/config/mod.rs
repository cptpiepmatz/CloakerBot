pub mod danger_zones;
pub mod immunity;
pub mod log_channel;
pub mod random;
pub mod target_bots;

use std::error::Error;
use std::fmt::{Display, Formatter};

use tokio::runtime::Handle;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::Interaction;
use twilight_util::builder::command::CommandBuilder;

use crate::interaction::application_command::{
    defer, HandleApplicationCommandError, TryFromInteractionError
};
use crate::util::impl_from_variant::impl_from_variant;

pub const NAME: &str = "config";
const DESCRIPTION: &str = "Configure the CloakerBot.";

pub enum ConfigCommand {
    DangerZones(danger_zones::DangerZonesSubCommand),
    LogChannel(log_channel::LogChannelSubCommand),
    Random(random::RandomSubCommand),
    TargetBots(target_bots::TargetBotsSubCommand),
    Immunity(immunity::ImmunitySubCommand)
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
                log_channel::LogChannelSubCommand::try_from((options, &data.resolved))?
            )),
            target_bots::NAME => Ok(ConfigCommand::TargetBots(
                target_bots::TargetBotsSubCommand::try_from(options)?
            )),
            random::NAME => Ok(ConfigCommand::Random(random::RandomSubCommand::try_from(
                options
            )?)),
            immunity::NAME => Ok(ConfigCommand::Immunity(
                immunity::ImmunitySubCommand::try_from(options)?
            )),
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
        .option(immunity::chat_input())
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

impl_from_variant!(
    ConfigActionError,
    HandleApplicationCommandError,
    ConfigAction
);

pub async fn action(
    interaction: &Interaction,
    command: ConfigCommand
) -> Result<(), HandleApplicationCommandError> {
    match command {
        ConfigCommand::DangerZones(cmd) => danger_zones::action(interaction, cmd).await?,
        ConfigCommand::LogChannel(cmd) => log_channel::action(interaction, cmd).await?,
        ConfigCommand::Random(cmd) => random::action(interaction, cmd).await?,
        ConfigCommand::TargetBots(cmd) => target_bots::action(interaction, cmd).await?,
        ConfigCommand::Immunity(cmd) => immunity::action(interaction, cmd).await?
    };

    Ok(())
}
