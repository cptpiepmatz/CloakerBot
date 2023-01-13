use std::sync::Arc;

use twilight_model::application::interaction::{Interaction, InteractionType};

use crate::interaction::application_command::handle_application_command;
use crate::Com;

pub mod application_command;

pub async fn handle_interaction(interaction: Interaction, com: Arc<Com>) -> anyhow::Result<()> {
    Ok(match interaction.kind {
        InteractionType::ApplicationCommand => handle_application_command(interaction, com).await?,
        _ => ()
    })
}
