use serde_json::json;
use twilight_http::request::application::interaction::CreateResponse;
use twilight_http::request::RequestBuilder;
use twilight_http::routing::Route;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, ComponentType};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::http::interaction::InteractionResponseType;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::SubCommandBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::application_command::config::ConfigActionError;
use crate::interaction::application_command::{
    HandleApplicationCommandError, TryFromInteractionError
};
use crate::interaction::message_component;
use crate::{APPLICATION_ID, STATE};

pub const NAME: &str = "danger-zones";
pub const DESCRIPTION: &str = "Define your danger zones.";

pub struct DangerZonesSubCommand;

impl TryFrom<&Vec<CommandDataOption>> for DangerZonesSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        if let Some(option) = options.iter().next() {
            return Err(TryFromInteractionError::UnknownField {
                command: format!("{} {}", super::NAME, NAME),
                field: String::from(&option.name)
            });
        }
        Ok(DangerZonesSubCommand)
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION).build()
}

pub async fn action(
    interaction: &Interaction,
    command: DangerZonesSubCommand
) -> Result<(), HandleApplicationCommandError> {
    let channel_select = message_component::channel_select::danger_zones::component();
    let button = message_component::button::config::component(
        message_component::button::config::ConfigUnsetButton::DangerZones
    );

    let payload = json!({
        "type": InteractionResponseType::ChannelMessageWithSource,
        "data": {
            "flags": MessageFlags::EPHEMERAL,
            "components": [
                {
                    "type": ComponentType::ActionRow,
                    "components": [channel_select]
                },
                {
                    "type": ComponentType::ActionRow,
                    "components": [button]
                }
            ]
        }
    });

    let request = RequestBuilder::new(&Route::InteractionCallback {
        interaction_id: interaction.id.into(),
        interaction_token: interaction.token.as_str()
    })
    .json(&payload)
    .expect("payload is serializable")
    .use_authorization_token(true)
    .build();

    STATE.http.request::<CreateResponse<'_>>(request).await?;

    Ok(())
}
