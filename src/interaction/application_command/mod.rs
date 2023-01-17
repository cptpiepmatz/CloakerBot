mod cloaker;
mod config;
mod counter;
mod help;
mod ranks;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
use twilight_http::response::marker::EmptyBody;
use twilight_http::Response;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::Message;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::application_command::cloaker::CloakerCommand;
use crate::interaction::application_command::config::ConfigCommand;
use crate::interaction::application_command::counter::CounterCommand;
use crate::interaction::application_command::help::HelpCommand;
use crate::interaction::application_command::ranks::RanksCommand;
use crate::{Com, APPLICATION_ID};

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

pub async fn register_commands(client: &InteractionClient<'_>) -> Result<(), TwilightHttpError> {
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

pub async fn handle_application_command(
    interaction: Interaction,
    com: Arc<Com>
) -> anyhow::Result<()> {
    match ApplicationCommand::try_from(&interaction)? {
        ApplicationCommand::Cloaker(command) => cloaker::action(interaction, command, com).await?,
        ApplicationCommand::Config(command) => config::action(interaction, command, com).await?,
        ApplicationCommand::Counter(_) => {}
        ApplicationCommand::Help(command) => help::action(interaction, command, com).await?,
        ApplicationCommand::Ranks(_) => {}
    };

    Ok(())
}

async fn defer(
    interaction: &Interaction,
    com: Arc<Com>
) -> Result<Response<EmptyBody>, twilight_http::Error> {
    let response = InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: Some(
            InteractionResponseDataBuilder::new()
                .flags(MessageFlags::EPHEMERAL)
                .build()
        )
    };
    com.http
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &response)
        .await
}

async fn update_defer(
    interaction: &Interaction,
    msg: &str,
    com: Arc<Com>
) -> anyhow::Result<Response<Message>> {
    Ok(com
        .http
        .interaction(APPLICATION_ID)
        .update_response(interaction.token.as_str())
        .content(Some(msg))?
        .await?)
}
