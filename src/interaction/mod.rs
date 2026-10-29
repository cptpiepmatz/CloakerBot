use derive_more::From;
use twilight_model::application::interaction::{Interaction, InteractionType};
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::HandleDiscordEventError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, handle_application_command
};
use crate::interaction::application_command_autocomplete::{
    HandleApplicationCommandAutocompleteError, handle_application_command_autocomplete
};
use crate::interaction::message_component::{
    HandleMessageComponentError, handle_message_component
};

mod application_command;
mod application_command_autocomplete;
mod message_component;

pub use application_command::{COMMANDS_REGISTER, CommandsRegister};

#[derive(Debug, From)]
pub enum HandleInteractionError {
    MessageComponent(HandleMessageComponentError),
    ApplicationCommand(HandleApplicationCommandError),
    ApplicationCommandAutocomplete(HandleApplicationCommandAutocompleteError)
}

pub async fn handle_interaction(
    interaction_create: &InteractionCreate
) -> Result<(), HandleInteractionError> {
    let interaction: &Interaction = interaction_create;
    match interaction.kind {
        InteractionType::MessageComponent => handle_message_component(interaction).await?,

        InteractionType::ApplicationCommand => handle_application_command(interaction).await?,
        InteractionType::ApplicationCommandAutocomplete => {
            handle_application_command_autocomplete(interaction).await?
        }

        _ => ()
    }

    Ok(())
}
