use derive_more::From;
use twilight_model::application::command::{CommandOptionChoice, CommandOptionChoiceValue};
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::application_command::help;
use crate::{APPLICATION_ID, STATE};

#[derive(Debug, From)]
pub enum HandleApplicationCommandAutocompleteError {
    Http(twilight_http::Error)
}

pub async fn handle_application_command_autocomplete(
    interaction: &Interaction
) -> Result<(), HandleApplicationCommandAutocompleteError> {
    let choices = match interaction.data.as_ref() {
        Some(InteractionData::ApplicationCommand(data)) if data.name == help::NAME => {
            data.options.iter().find_map(|option| match (&*option.name, &option.value) {
                ("command", CommandOptionValue::Focused(query, _)) => Some(query.to_lowercase()),
                _ => None
            }).map(|query| help::CHOICES.iter()
                .filter(|name| name.contains(&query))
                .take(25)
                .map(|name| CommandOptionChoice {
                    name: (*name).into(),
                    name_localizations: None,
                    value: CommandOptionChoiceValue::String((*name).into())
                }).collect::<Vec<_>>()).unwrap_or_default()
        }
        _ => Vec::new()
    };
    STATE.client.interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
            data: Some(InteractionResponseDataBuilder::new().choices(choices).build())
        }).await?;
    Ok(())
}
