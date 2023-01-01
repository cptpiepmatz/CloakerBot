use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;
use twilight_http::client::InteractionClient;
use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::{Interaction, InteractionType};

use crate::interactions::commands::handle_application_command;
use crate::{Com, SharedData};

pub mod commands;

pub async fn handle_interaction(interaction: Interaction, com: Arc<Com>) {
    match interaction.kind {
        InteractionType::ApplicationCommand => handle_application_command(interaction, com).await,
        _ => ()
    }
}
