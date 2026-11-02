use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::message_component::HandleMessageComponentError;
use crate::{APPLICATION_ID, STATE};

pub async fn disable(interaction: &Interaction) -> Result<(), HandleMessageComponentError> {
    let owner_id = interaction.author_id().ok_or(HandleMessageComponentError::NoAuthor)?;
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new().flags(MessageFlags::EPHEMERAL).build()
            )
        })
        .await?;

    {
        let mut notifications = STATE.owner_notifications.lock().await;
        STATE.database.set_owner_notification_allowance(owner_id, false).await?;
        notifications.remove(&owner_id);
    }

    STATE
        .client
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .content(Some("Owner notifications are now disabled. You won't receive future updates."))
        .await?;
    Ok(())
}
