use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, Separator, SeparatorSpacingSize, TextDisplay
};

use crate::assets::templates::owner_notifications;
use crate::interaction::defer;
use crate::interaction::message_component::HandleMessageComponentError;
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::{APPLICATION_ID, STATE};

pub async fn disable(interaction: &Interaction) -> Result<(), HandleMessageComponentError> {
    set_allowance(interaction, false).await
}

pub async fn enable(interaction: &Interaction) -> Result<(), HandleMessageComponentError> {
    set_allowance(interaction, true).await
}

async fn set_allowance(
    interaction: &Interaction,
    allow_dms: bool
) -> Result<(), HandleMessageComponentError> {
    let owner_id = interaction.author_id().ok_or(HandleMessageComponentError::NoAuthor)?;
    defer::public(interaction).await?;

    {
        let mut notifications = STATE.owner_notifications.lock().await;
        STATE.database.set_owner_notification_allowance(owner_id, allow_dms).await?;
        notifications.remove(&owner_id);
    }

    let (content, custom_id, label) = if allow_dms {
        (
            owner_notifications::enabled().to_string(),
            MessageComponentCustomId::OwnerNotificationsDisable,
            owner_notifications::disable_button().to_string()
        )
    }
    else {
        (
            owner_notifications::disabled().to_string(),
            MessageComponentCustomId::OwnerNotificationsEnable,
            owner_notifications::enable_button().to_string()
        )
    };

    let components = [
        Component::TextDisplay(TextDisplay { id: None, content }),
        Component::Separator(Separator {
            id: None,
            divider: Some(false),
            spacing: Some(SeparatorSpacingSize::Small)
        }),
        Component::ActionRow(ActionRow {
            id: None,
            components: vec![Component::Button(Button {
                id: None,
                style: ButtonStyle::Secondary,
                custom_id: Some(custom_id.to_string()),
                label: Some(label),
                disabled: false,
                emoji: None,
                url: None,
                sku_id: None
            })]
        })
    ];

    STATE
        .client
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .flags(MessageFlags::IS_COMPONENTS_V2)
        .components(Some(&components))
        .await?;
    Ok(())
}
