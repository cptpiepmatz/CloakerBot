use derive_more::From;
use twilight_model::application::interaction::{Interaction, InteractionData};

use crate::interaction::message_component::button::config::ConfigButtonActionError;
use crate::interaction::message_component::channel_select::danger_zones::DangerZonesSelectActionError;
use crate::interaction::message_component::channel_select::log_channel::LogChannelSelectActionError;
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::interaction::message_component::role_select::immunity::ImmunitySelectActionError;
use crate::interaction::HandleInteractionError;

pub mod button;
pub mod channel_select;
mod custom_id;
pub mod role_select;

#[derive(Debug, From)]
pub enum HandleMessageComponentError {
    NoMessageComponent,
    UnknownCustomId(String),

    ConfigButtonAction(ConfigButtonActionError),
    DangerZonesSelectAction(DangerZonesSelectActionError),
    ImmunitySelectAction(ImmunitySelectActionError),
    LogChannelSelectAction(LogChannelSelectActionError),

    Http(twilight_http::Error)
}

pub async fn handle_message_component(
    interaction: &Interaction
) -> Result<(), HandleMessageComponentError> {
    let Some(InteractionData::MessageComponent(data)) = interaction.data.as_ref()
    else {
        return Err(HandleMessageComponentError::NoMessageComponent);
    };

    let custom_id: MessageComponentCustomId = data.custom_id.parse()?;
    use MessageComponentCustomId::*;
    match custom_id {
        ConfigUnsetDangerZones |
        ConfigUnsetImmunityGeneral |
        ConfigUnsetImmunityRandom |
        ConfigUnsetImmunityGeneralSilent |
        ConfigUnsetImmunityRandomSilent |
        ConfigUnsetLogChannel |
        ConfigUnsetRandom => button::config::action(interaction, custom_id).await,

        DangerZonesSelect => channel_select::danger_zones::action(interaction, data).await,
        ImmunityGeneralSelect |
        ImmunityRandomSelect |
        ImmunityGeneralSilentSelect |
        ImmunityRandomSilentSelect => {
            role_select::immunity::action(interaction, custom_id, data).await
        }
        LogChannelSelect => channel_select::log_channel::action(interaction, data).await
    }
}
