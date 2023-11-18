use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use derive_more::From;
use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
use twilight_http::response::DeserializeBodyError;
use twilight_mention::fmt::CommandMention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_validate::embed::EmbedValidationError;
use twilight_validate::message::MessageValidationError;

use crate::interaction::application_command::cloaker::{CloakerActionError, CloakerCommand};
use crate::interaction::application_command::config::{ConfigActionError, ConfigCommand};
use crate::interaction::application_command::counter::CounterCommand;
use crate::interaction::application_command::help::HelpCommand;
use crate::interaction::application_command::ranks::RanksCommand;
use crate::interaction::HandleInteractionError;
use crate::sync::OnceLockExpect;
use crate::{APPLICATION_ID, STATE};

mod cloaker;
mod config;
mod counter;
mod help;
mod ranks;

enum ApplicationCommand {
    Cloaker(CloakerCommand),
    Config(ConfigCommand),
    Counter(CounterCommand),
    Help(HelpCommand),
    Ranks(RanksCommand)
}

impl TryFrom<&Interaction> for ApplicationCommand {
    type Error = TryFromInteractionError;

    fn try_from(interaction: &Interaction) -> Result<Self, Self::Error> {
        let Some(InteractionData::ApplicationCommand(data)) = &interaction.data
        else {
            return Err(TryFromInteractionError::NoApplicationCommand);
        };

        match data.name.as_str() {
            cloaker::NAME => Ok(Self::Cloaker(CloakerCommand::try_from(data.as_ref())?)),
            config::NAME => Ok(Self::Config(ConfigCommand::try_from(data.as_ref())?)),
            counter::NAME => Ok(Self::Counter(CounterCommand::try_from(data.as_ref())?)),
            help::NAME => Ok(Self::Help(HelpCommand::try_from(data.as_ref())?)),
            ranks::NAME => Ok(Self::Ranks(RanksCommand::try_from(data.as_ref())?)),
            _ => Err(TryFromInteractionError::UnknownCommandName {
                name: data.name.clone()
            })
        }
    }
}

#[derive(Debug)]
pub enum TryFromInteractionError {
    NoApplicationCommand,
    UnknownCommandName {
        name: String
    },
    MissingField {
        command: String,
        field: String
    },
    IncorrectFieldType {
        command: String,
        field: String,
        value: CommandOptionValue
    },
    IncorrectFieldValue {
        command: String,
        field: String,
        value: CommandOptionValue
    },
    UnknownField {
        command: String,
        field: String
    }
}

impl Display for TryFromInteractionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TryFromInteractionError::NoApplicationCommand => write!(f, "no application command"),
            TryFromInteractionError::UnknownCommandName { name } => {
                write!(f, "unknown command {}", name)
            }
            TryFromInteractionError::MissingField { command, field } => {
                write!(f, "missing field {} in {}", field, command)
            }
            TryFromInteractionError::IncorrectFieldType {
                command,
                field,
                value
            } => write!(
                f,
                "incorrect field type of {} in {}, got {:?}",
                field, command, value
            ),
            TryFromInteractionError::IncorrectFieldValue {
                command,
                field,
                value
            } => write!(
                f,
                "incorrect field value of {} in {}, got {:?}",
                field, command, value
            ),
            TryFromInteractionError::UnknownField { command, field } => {
                write!(f, "unknown field {} in {}", field, command)
            }
        }
    }
}

impl Error for TryFromInteractionError {}

#[derive(Debug, From)]
pub enum RegisterCommandsError {
    Http(TwilightHttpError),
    DeserializeBody(DeserializeBodyError),
    MissingRegistries(Vec<Command>)
}

pub async fn register_commands(
    client: &InteractionClient<'_>
) -> Result<(), RegisterCommandsError> {
    let commands = client
        .set_global_commands(&[
            cloaker::chat_input(),
            cloaker::user_context_menu(),
            cloaker::message_context_menu(),
            counter::chat_input(),
            counter::user_context_menu(),
            counter::message_context_menu(),
            config::chat_input(),
            ranks::chat_input(),
            help::chat_input()
        ])
        .await?
        .models()
        .await?;

    let mut command_map = HashMap::new();
    for command in commands.iter() {
        // we only care for chat input IDs
        if command.kind != CommandType::ChatInput {
            continue;
        }

        // insert newest version of a command into the map
        let mapped_command = command_map
            .entry(command.name.as_str())
            .or_insert(command.clone());
        if mapped_command.version.into_nonzero() < command.version.into_nonzero() {
            command_map.insert(command.name.as_str(), command.clone());
        }
    }

    for (command_name, command) in command_map {
        if let None = command.guild_id {
            match (command_name, command.id) {
                (config::NAME, Some(command_id)) => {
                    OnceLockExpect::set(&config::ID, command_id);
                    for (mention, name) in [
                        (&config::danger_zones::MENTION, config::danger_zones::NAME),
                        (&config::immunity::MENTION, config::immunity::NAME),
                        (&config::log_channel::MENTION, config::log_channel::NAME),
                        (&config::random::MENTION, config::random::NAME)
                    ] {
                        OnceLockExpect::set(
                            mention,
                            CommandMention::SubCommand {
                                name: config::NAME.to_owned(),
                                sub_command: name.to_owned(),
                                id: command_id
                            }
                            .into_mention()
                        );
                    }
                }
                (cloaker::NAME, Some(command_id)) => {
                    OnceLockExpect::set(&cloaker::ID, command_id);
                    OnceLockExpect::set(
                        &cloaker::MENTION,
                        CommandMention::Command {
                            name: cloaker::NAME.to_owned(),
                            id: command_id
                        }
                        .into_mention()
                    );
                }
                (counter::NAME, Some(command_id)) => {
                    OnceLockExpect::set(&counter::ID, command_id);
                    OnceLockExpect::set(
                        &counter::MENTION,
                        CommandMention::Command {
                            name: counter::NAME.to_owned(),
                            id: command_id
                        }
                        .into_mention()
                    );
                }
                (help::NAME, Some(command_id)) => {
                    OnceLockExpect::set(&help::ID, command_id);
                    OnceLockExpect::set(
                        &help::MENTION,
                        CommandMention::Command {
                            name: help::NAME.to_owned(),
                            id: command_id
                        }
                        .into_mention()
                    );
                }
                (ranks::NAME, Some(command_id)) => {
                    OnceLockExpect::set(&ranks::ID, command_id);
                    OnceLockExpect::set(
                        &ranks::MENTION,
                        CommandMention::Command {
                            name: ranks::NAME.to_owned(),
                            id: command_id
                        }
                        .into_mention()
                    );
                }
                _ => ()
            }
        }
    }

    match OnceLockExpect::is_initialized(&config::ID) &&
        OnceLockExpect::is_initialized(&config::danger_zones::MENTION) &&
        OnceLockExpect::is_initialized(&config::immunity::MENTION) &&
        OnceLockExpect::is_initialized(&config::log_channel::MENTION) &&
        OnceLockExpect::is_initialized(&config::random::MENTION) &&
        OnceLockExpect::is_initialized(&cloaker::ID) &&
        OnceLockExpect::is_initialized(&cloaker::MENTION) &&
        OnceLockExpect::is_initialized(&counter::ID) &&
        OnceLockExpect::is_initialized(&counter::MENTION) &&
        OnceLockExpect::is_initialized(&help::ID) &&
        OnceLockExpect::is_initialized(&help::MENTION) &&
        OnceLockExpect::is_initialized(&ranks::ID) &&
        OnceLockExpect::is_initialized(&ranks::MENTION)
    {
        true => Ok(()),
        false => Err(RegisterCommandsError::MissingRegistries(commands))
    }
}

#[derive(Debug, From)]
pub enum HandleApplicationCommandError {
    TryFromInteraction(TryFromInteractionError),

    ConfigAction(ConfigActionError),
    CloakerAction(CloakerActionError),

    Defer(DeferError),
    UpdateDefer(UpdateDeferError),

    Http(twilight_http::Error)
}

pub async fn handle_application_command(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandError> {
    match ApplicationCommand::try_from(interaction)? {
        ApplicationCommand::Cloaker(command) => cloaker::action(interaction, command).await,
        ApplicationCommand::Config(command) => config::action(interaction, command).await,
        ApplicationCommand::Counter(_) => todo!(),
        ApplicationCommand::Help(command) => help::action(interaction, command).await,
        ApplicationCommand::Ranks(_) => todo!()
    }
}

#[derive(Debug)]
pub struct DeferError(twilight_http::Error);

impl From<twilight_http::Error> for DeferError {
    fn from(value: TwilightHttpError) -> Self {
        DeferError(value)
    }
}

async fn defer(interaction: &Interaction) -> Result<(), DeferError> {
    let response = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .flags(MessageFlags::EPHEMERAL)
                .build()
        )
    };
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &response)
        .await?;

    Ok(())
}

#[derive(Debug, From)]
pub enum UpdateDeferError {
    Http(twilight_http::Error),
    MessageValidation(MessageValidationError),
    EmbedValidation(EmbedValidationError)
}

async fn update_defer_msg(interaction: &Interaction, msg: &str) -> Result<(), UpdateDeferError> {
    STATE
        .client
        .interaction(APPLICATION_ID)
        .update_response(interaction.token.as_str())
        .content(Some(msg))?
        .await?;

    Ok(())
}

async fn update_defer_embeds(
    interaction: &Interaction,
    embed_builder: EmbedBuilder
) -> Result<(), UpdateDeferError> {
    STATE
        .client
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .embeds(Some(&[embed_builder.validate()?.build()]))?
        .await?;

    Ok(())
}
