use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::LazyLock;

use derive_more::From;
use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
use twilight_http::response::DeserializeBodyError;
use twilight_mention::fmt::CommandMention;
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_validate::embed::EmbedValidationError;
use twilight_validate::message::MessageValidationError;

use crate::diagnostics::panic::ExpectReport;
use crate::interaction::application_command::cloaker::{CloakerActionError, CloakerCommand};
use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::counter::{CounterActionError, CounterCommand};
use crate::interaction::application_command::help::HelpCommand;
use crate::interaction::application_command::ranks::{RanksActionError, RanksCommand};
use crate::interaction::{DeferError, HandleInteractionError};
use crate::{APPLICATION_ID, STATE};

pub(super) mod cloaker;
pub(super) mod config;
pub(super) mod counter;
pub(super) mod help;
pub(super) mod ranks;

enum ApplicationCommand {
    Cloaker(CloakerCommand),
    Config,
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
            config::NAME => Ok(Self::Config),
            counter::NAME => Ok(Self::Counter(CounterCommand::try_from(data.as_ref())?)),
            help::NAME => Ok(Self::Help(HelpCommand::try_from(data.as_ref())?)),
            ranks::NAME => Ok(Self::Ranks(RanksCommand::try_from(data.as_ref())?)),
            _ => Err(TryFromInteractionError::UnknownCommandName { name: data.name.clone() })
        }
    }
}

#[derive(Debug)]
pub enum TryFromInteractionError {
    NoApplicationCommand,
    UnknownCommandName { name: String },
    MissingField { command: String, field: String },
    IncorrectFieldType { command: String, field: String, value: CommandOptionValue },
    IncorrectFieldValue { command: String, field: String, value: CommandOptionValue },
    UnknownField { command: String, field: String }
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
            TryFromInteractionError::IncorrectFieldType { command, field, value } => {
                write!(f, "incorrect field type of {} in {}, got {:?}", field, command, value)
            }
            TryFromInteractionError::IncorrectFieldValue { command, field, value } => {
                write!(f, "incorrect field value of {} in {}, got {:?}", field, command, value)
            }
            TryFromInteractionError::UnknownField { command, field } => {
                write!(f, "unknown field {} in {}", field, command)
            }
        }
    }
}

impl Error for TryFromInteractionError {}

pub struct CommandsRegister {
    config: Id<CommandMarker>,
    cloaker: Id<CommandMarker>,
    counter: Id<CommandMarker>,
    help: Id<CommandMarker>,
    ranks: Id<CommandMarker>
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterCommandsError {
    #[error("could not register commands with Discord")]
    Http(#[from] TwilightHttpError),

    #[error("could not decode registered commands")]
    DeserializeBody(#[from] DeserializeBodyError),

    #[error("Discord did not return all required command registrations: {0:?}")]
    MissingRegistries(Vec<Command>)
}

pub static COMMANDS_REGISTER: LazyLock<CommandsRegister> = LazyLock::new(|| {
    let client = STATE.client.interaction(APPLICATION_ID);

    let commands = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async {
                Result::<_, RegisterCommandsError>::Ok(
                    client
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
                        .await?
                )
            })
            .expect_report("could not register commands")
    });

    let mut command_map = HashMap::new();
    for command in commands.iter() {
        // we only care for chat input IDs
        if command.kind != CommandType::ChatInput {
            continue;
        }

        let mapped_command =
            command_map.entry(command.name.as_str()).or_insert_with(|| command.clone());
        if mapped_command.version.into_nonzero() < command.version.into_nonzero() {
            command_map.insert(command.name.as_str(), command.clone());
        }
    }

    let mut config: Option<Id<CommandMarker>> = None;
    let mut cloaker: Option<Id<CommandMarker>> = None;
    let mut counter: Option<Id<CommandMarker>> = None;
    let mut help: Option<Id<CommandMarker>> = None;
    let mut ranks: Option<Id<CommandMarker>> = None;

    for (command_name, command) in command_map {
        if command.guild_id.is_some() {
            continue;
        }

        match (command_name, command.id) {
            (config::NAME, id @ Some(_)) => config = id,
            (cloaker::NAME, id @ Some(_)) => cloaker = id,
            (counter::NAME, id @ Some(_)) => counter = id,
            (help::NAME, id @ Some(_)) => help = id,
            (ranks::NAME, id @ Some(_)) => ranks = id,
            _ => {}
        }
    }

    let (Some(config), Some(cloaker), Some(counter), Some(help), Some(ranks)) =
        (config, cloaker, counter, help, ranks)
    else {
        return Result::<CommandsRegister, _>::Err(RegisterCommandsError::MissingRegistries(
            commands
        ))
        .expect_report("could not init command registry");
    };

    CommandsRegister { config, cloaker, counter, help, ranks }
});

macro_rules! mention {
    (command) => {{
        use twilight_mention::Mention;
        LazyLock::new(|| CommandMention::Command { id: *ID, name: NAME.to_string() }.mention())
    }};

    (subcommand) => {{
        use twilight_mention::Mention;
        LazyLock::new(|| {
            CommandMention::SubCommand {
                id: super::super::COMMANDS_REGISTER.config,
                name: super::NAME.to_string(),
                sub_command: NAME.to_string()
            }
            .mention()
        })
    }};
}
use mention;

#[derive(Debug, From)]
pub enum HandleApplicationCommandError {
    Database(sqlx::Error),
    TryFromInteraction(TryFromInteractionError),

    ConfigAction(ConfigActionError),
    CloakerAction(CloakerActionError),
    CounterAction(CounterActionError),
    RanksAction(RanksActionError),

    Defer(DeferError),
    UpdateDefer(UpdateDeferError),

    Http(twilight_http::Error),
    DeserializeBody(DeserializeBodyError)
}

pub async fn handle_application_command(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandError> {
    match ApplicationCommand::try_from(interaction)? {
        ApplicationCommand::Cloaker(command) => cloaker::action(interaction, command).await,
        ApplicationCommand::Config => config::action(interaction).await,
        ApplicationCommand::Counter(command) => counter::action(interaction, command).await,
        ApplicationCommand::Help(command) => help::action(interaction, command).await,
        ApplicationCommand::Ranks(command) => ranks::action(interaction, command).await
    }
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
        .content(Some(msg))
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
        .embeds(Some(&[embed_builder.validate()?.build()]))
        .await?;

    Ok(())
}
