use derive_more::From;
use twilight_model::application::interaction::Interaction;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::{APPLICATION_ID, STATE};

#[derive(Debug, From)]
pub enum HandleApplicationCommandAutocompleteError {
    Http(twilight_http::Error)
}

pub async fn handle_application_command_autocomplete(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandAutocompleteError> {
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
            data: Some(InteractionResponseDataBuilder::new().choices(Vec::new()).build())
        })
        .await?;
    Ok(())
}
