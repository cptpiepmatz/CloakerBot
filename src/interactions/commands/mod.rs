mod cloaker;
mod config;
mod counter;
mod help;
mod ranks;

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::id::Id;

use crate::interactions::commands::cloaker::CloakerCommand;
use crate::interactions::commands::config::ConfigCommand;
use crate::interactions::commands::counter::CounterCommand;
use crate::interactions::commands::help::HelpCommand;
use crate::interactions::commands::ranks::RanksCommand;
use crate::Com;

enum ApplicationCommand {
    Cloaker(CloakerCommand),
    Config(ConfigCommand),
    Counter(CounterCommand),
    Help(HelpCommand),
    Ranks(RanksCommand)
}

impl TryFrom<&mut Interaction> for ApplicationCommand {
    type Error = TryFromInteractionError;

    fn try_from(interaction: &mut Interaction) -> Result<Self, Self::Error> {
        let Some(InteractionData::ApplicationCommand(data)) = &mut interaction.data else {
            return Err(TryFromInteractionError::NoApplicationCommand);
        };

        match data.name.as_str() {
            cloaker::NAME => Ok(Self::Cloaker(CloakerCommand::try_from(data.as_mut())?)),
            config::NAME => Ok(Self::Config(ConfigCommand::try_from(data.as_mut())?)),
            counter::NAME => Ok(Self::Counter(CounterCommand::try_from(data.as_mut())?)),
            help::NAME => Ok(Self::Help(HelpCommand::try_from(data.as_mut())?)),
            ranks::NAME => Ok(Self::Ranks(RanksCommand::try_from(data.as_mut())?)),
            _ => Err(TryFromInteractionError::UnknownCommandName { name: data.name })
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
    UnknownField {
        command: String,
        field: String
    }
}

impl Display for TryFromInteractionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
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

pub async fn handle_application_command(interaction: Interaction, com: Arc<Com>) {
    if let Some(InteractionData::ApplicationCommand(data)) = &interaction.data {
        match data.name.as_str() {
            cloaker::NAME => cloaker::action(interaction, com).await,
            config::NAME => {}
            counter::NAME => {}
            help::NAME => {}
            ranks::NAME => {}
            _ => {}
        }
    }
}
