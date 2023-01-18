use std::sync::Arc;

use twilight_model::application::interaction::{Interaction, InteractionType};

use crate::interaction::application_command::handle_application_command;
use crate::interaction::message_component::handle_message_component;
use crate::State;

pub mod application_command;
pub mod message_component;

pub async fn handle_interaction(interaction: Interaction, com: Arc<State>) -> anyhow::Result<()> {
    match interaction.kind {
        InteractionType::ApplicationCommand => handle_application_command(interaction, com).await?,
        InteractionType::MessageComponent => handle_message_component(interaction, com).await?,
        _ => ()
    };

    Ok(())
}
