use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
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
use crate::{impl_from_variant, APPLICATION_ID, STATE};

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
        let Some(InteractionData::ApplicationCommand(data)) = &interaction.data else {
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

impl_from_variant!(
    TryFromInteractionError,
    HandleApplicationCommandError,
    TryFromInteraction
);

pub async fn register_commands(client: &InteractionClient<'_>) -> Result<(), TwilightHttpError> {
    // TODO: register commands globally instead
    let _res = client
        .set_guild_commands(Id::new(581884294495207444), &[
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
        .await?;
    Ok(())
}

#[derive(Debug)]
pub enum HandleApplicationCommandError {
    TryFromInteraction(TryFromInteractionError),

    ConfigAction(ConfigActionError),
    CloakerAction(CloakerActionError),

    Defer(DeferError),
    UpdateDefer(UpdateDeferError)
}

impl_from_variant!(
    HandleApplicationCommandError,
    HandleInteractionError,
    ApplicationCommand
);

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

impl_from_variant!(DeferError, HandleApplicationCommandError, Defer);

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
        .http
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &response)
        .await?;

    Ok(())
}

#[derive(Debug)]
pub enum UpdateDeferError {
    Http(twilight_http::Error),
    MessageValidation(MessageValidationError),
    EmbedValidation(EmbedValidationError)
}

impl_from_variant!(twilight_http::Error, UpdateDeferError, Http);
impl_from_variant!(MessageValidationError, UpdateDeferError, MessageValidation);
impl_from_variant!(EmbedValidationError, UpdateDeferError, EmbedValidation);
impl_from_variant!(UpdateDeferError, HandleApplicationCommandError, UpdateDefer);

async fn update_defer_msg(interaction: &Interaction, msg: &str) -> Result<(), UpdateDeferError> {
    STATE
        .http
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
        .http
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .embeds(Some(&[embed_builder.validate()?.build()]))?
        .await?;

    Ok(())
}
