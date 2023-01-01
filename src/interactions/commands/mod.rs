mod cloaker;
mod config;
mod counter;
mod help;
mod ranks;

use std::sync::Arc;

use twilight_http::client::InteractionClient;
use twilight_http::error::Error as TwilightHttpError;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::id::Id;

use crate::Com;

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
