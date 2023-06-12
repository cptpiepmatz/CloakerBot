use std::str::FromStr;

use twilight_model::application::interaction::{Interaction, InteractionData};

use crate::interaction::message_component::button::config::ConfigActionError;
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::interaction::HandleInteractionError;
use crate::util::impl_from_variant::impl_from_variant;

pub mod button;
mod custom_id;

#[derive(Debug)]
pub enum HandleMessageComponentError {
    NoMessageComponent,
    UnknownCustomId(String),

    ConfigAction(ConfigActionError),

    Http(twilight_http::Error)
}

impl_from_variant!(
    HandleMessageComponentError,
    HandleInteractionError,
    MessageComponent
);

impl_from_variant!(twilight_http::Error, HandleMessageComponentError, Http);

pub async fn handle_message_component(
    interaction: &Interaction
) -> Result<(), HandleMessageComponentError> {
    let Some(InteractionData::MessageComponent(data)) = interaction.data.as_ref() else {
        return Err(HandleMessageComponentError::NoMessageComponent);
    };

    let custom_id: MessageComponentCustomId = data.custom_id.parse()?;
    use MessageComponentCustomId::*;
    match custom_id {
        ConfigUnsetDangerZones => todo!(),
        ConfigUnsetImmunityGeneral => todo!(),
        ConfigUnsetImmunityRandom => todo!(),
        ConfigUnsetLogChannel => todo!(),
        ConfigUnsetRandom => todo!(),
        ConfigUnsetTargetBots => button::config::action(interaction, custom_id).await
    }
}
