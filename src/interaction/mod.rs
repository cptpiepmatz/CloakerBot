use derive_more::From;
use twilight_model::application::interaction::{Interaction, InteractionType};
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::interaction::application_command::{
    handle_application_command, HandleApplicationCommandError
};
use crate::interaction::application_command_autocomplete::{
    handle_application_command_autocomplete, HandleApplicationCommandAutocompleteError
};
use crate::interaction::message_component::{
    handle_message_component, HandleMessageComponentError
};
use crate::HandleDiscordEventError;

mod application_command;
mod application_command_autocomplete;
mod message_component;

pub use application_command::register_commands;

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
            handle_application_command_autocomplete(interaction)?
        }

        _ => ()
    }

    Ok(())
}
