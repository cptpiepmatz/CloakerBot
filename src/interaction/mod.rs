use derive_more::From;
use twilight_model::application::interaction::{Interaction, InteractionData, InteractionType};
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
pub(crate) mod message_component;
mod modal_submit;

pub use application_command::{COMMANDS_REGISTER, CommandsRegister};

pub mod command_mentions {
    pub use super::application_command::{
        cloaker::MENTION as CLOAKER, config::MENTION as CONFIG, counter::MENTION as COUNTER,
        help::MENTION as HELP, ranks::MENTION as RANKS
    };
}

#[derive(Debug, From)]
pub enum HandleInteractionError {
    MessageComponent(HandleMessageComponentError),
    ModalSubmit(modal_submit::HandleModalSubmitError),
    ApplicationCommand(HandleApplicationCommandError),
    ApplicationCommandAutocomplete(HandleApplicationCommandAutocompleteError)
}

#[tracing::instrument(skip_all, fields(
    interaction_id = %interaction_create.id,
    guild_id = interaction_create.guild_id.map(|id| id.get()),
    user_id = interaction_create.author_id().map(|id| id.get()),
    interaction_kind = ?interaction_create.kind,
    command_name,
    component_id,
    modal_id
), err(Debug))]
pub async fn handle_interaction(
    interaction_create: &InteractionCreate
) -> Result<(), HandleInteractionError> {
    let interaction: &Interaction = interaction_create;
    let span = tracing::Span::current();
    match interaction.data.as_ref() {
        Some(InteractionData::ApplicationCommand(data)) => {
            span.record("command_name", data.name.as_str());
        }
        Some(InteractionData::MessageComponent(data)) => {
            span.record("component_id", data.custom_id.as_str());
        }
        Some(InteractionData::ModalSubmit(data)) => {
            span.record("modal_id", data.custom_id.as_str());
        }
        _ => {}
    }
    tracing::debug!("handling interaction");
    match interaction.kind {
        InteractionType::MessageComponent => handle_message_component(interaction).await?,
        InteractionType::ModalSubmit => modal_submit::handle_modal_submit(interaction).await?,
        InteractionType::ApplicationCommand => handle_application_command(interaction).await?,
        InteractionType::ApplicationCommandAutocomplete => {
            handle_application_command_autocomplete(interaction).await?
        }

        _ => ()
    }

    Ok(())
}
