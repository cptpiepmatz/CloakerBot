use std::str::FromStr;

use twilight_model::application::interaction::{Interaction, InteractionData};

use crate::interaction::message_component::button::config::ConfigButtonActionError;
use crate::interaction::message_component::channel_select::danger_zones::DangerZonesSelectActionError;
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::interaction::HandleInteractionError;
use crate::util::impl_from_variant::impl_from_variant;

pub mod button;
pub mod channel_select;
mod custom_id;

#[derive(Debug)]
pub enum HandleMessageComponentError {
    NoMessageComponent,
    UnknownCustomId(String),

    ConfigButtonAction(ConfigButtonActionError),
    DangerZonesSelectAction(DangerZonesSelectActionError),

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
        ConfigUnsetDangerZones |
        ConfigUnsetImmunityGeneral |
        ConfigUnsetImmunityRandom |
        ConfigUnsetLogChannel |
        ConfigUnsetRandom |
        ConfigUnsetTargetBots => button::config::action(interaction, custom_id).await,

        DangerZonesSelect => channel_select::danger_zones::action(interaction, data).await
    }
}
