pub mod danger_zones;
pub mod immunity;
pub mod log_channel;
pub mod random;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;

use thiserror::Error;
use tokio::runtime::Handle;
use twilight_http::request::Request;
use twilight_http::response::marker::EmptyBody;
use twilight_http::routing::Route;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::{
    CommandData, CommandDataOption, CommandOptionValue
};
use twilight_model::application::interaction::{Interaction, InteractionContextType};
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType
};
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::command::CommandBuilder;

use super::COMMANDS_REGISTER;
use crate::STATE;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError, defer
};

pub const NAME: &str = "config";
pub const DESCRIPTION: &str = "Configure the CloakerBot.";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.config);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

pub enum ConfigCommand {
    DangerZones(danger_zones::DangerZonesSubCommand),
    LogChannel(log_channel::LogChannelSubCommand),
    Random(random::RandomSubCommand),
    Immunity(immunity::ImmunitySubCommand)
}

impl TryFrom<&CommandData> for ConfigCommand {
    type Error = TryFromInteractionError;

    fn try_from(data: &CommandData) -> Result<Self, Self::Error> {
        let Some(CommandDataOption { name, value }) = data.options.first()
        else {
            return Err(TryFromInteractionError::MissingField {
                command: NAME.into(),
                field: "subcommand".into()
            });
        };

        let CommandOptionValue::SubCommand(options) = value
        else {
            return Err(TryFromInteractionError::IncorrectFieldType {
                command: NAME.into(),
                field: "subcommand".into(),
                value: value.clone()
            });
        };

        match name.as_str() {
            danger_zones::NAME => Ok(ConfigCommand::DangerZones(
                danger_zones::DangerZonesSubCommand::try_from(options)?
            )),
            log_channel::NAME => {
                Ok(ConfigCommand::LogChannel(log_channel::LogChannelSubCommand::try_from(options)?))
            }
            random::NAME => Ok(ConfigCommand::Random(random::RandomSubCommand::try_from(options)?)),
            immunity::NAME => {
                Ok(ConfigCommand::Immunity(immunity::ImmunitySubCommand::try_from(options)?))
            }
            n => Err(TryFromInteractionError::UnknownCommandName { name: format!("{NAME} {}", n) })
        }
    }
}

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .contexts([InteractionContextType::Guild])
        .option(log_channel::chat_input())
        .option(danger_zones::chat_input())
        .option(random::chat_input())
        .option(immunity::chat_input())
        .build()
}

#[derive(Debug, Error)]
pub enum ConfigActionError {
    #[error("not in guild")]
    NotInGuild,
    #[error("could not serialize config modal")]
    Serialize(#[from] serde_json::Error)
}

async fn respond_with_modal(
    interaction: &Interaction,
    data: InteractionResponseData
) -> Result<(), HandleApplicationCommandError> {
    let mut response = serde_json::to_value(InteractionResponse {
        kind: InteractionResponseType::Modal,
        data: Some(data)
    })
    .map_err(ConfigActionError::from)?;
    // Twilight 0.17 emits disabled on selects, but Discord forbids it in modals
    if let Some(components) = response["data"]["components"].as_array_mut() {
        for label in components {
            if let Some(component) = label["component"].as_object_mut() {
                component.remove("disabled");
            }
        }
    }
    let request = Request::builder(&Route::InteractionCallback {
        interaction_id: interaction.id.get(),
        interaction_token: &interaction.token,
        with_response: false
    })
    .use_authorization_token(false)
    .json(&response)
    .build()?;
    STATE.client.request::<EmptyBody>(request).await?;
    Ok(())
}

pub async fn action(
    interaction: &Interaction,
    command: ConfigCommand
) -> Result<(), HandleApplicationCommandError> {
    match command {
        ConfigCommand::DangerZones(cmd) => danger_zones::action(interaction, cmd).await?,
        ConfigCommand::LogChannel(cmd) => log_channel::action(interaction, cmd).await?,
        ConfigCommand::Random(cmd) => random::action(interaction, cmd).await?,
        ConfigCommand::Immunity(cmd) => immunity::action(interaction, cmd).await?
    };

    Ok(())
}
