use std::collections::hash_map::Entry;

use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_http::error::ErrorType;
use twilight_model::channel::Message;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::{
    ActionRow, Button, ButtonStyle, Component, TextDisplay
};

use crate::STATE;
use crate::data::owner_notifications::OwnerNotified;
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::util::ClientError;

const MAX_MESSAGE_TEXT_LENGTH: usize = 4000;
const MAX_MESSAGE_COMPONENTS: usize = 40;

#[derive(Debug, derive_more::From)]
pub enum OwnerNotificationError {
    Client(ClientError),
    Database(sqlx::Error),
    SectionTooLong(usize)
}

pub async fn on_message(message: &Message) -> Result<(), OwnerNotificationError> {
    let Some(guild_id) = message.guild_id
    else {
        return Ok(());
    };
    if message.author.bot {
        return Ok(());
    }
    let owner_id = match STATE.cache.guild(guild_id) {
        Some(guild) => guild.owner_id(),
        None => return Ok(())
    };
    if message.author.id != owner_id {
        return Ok(());
    }

    let mut notifications = STATE.owner_notifications.lock().await;
    let notified = match notifications.entry(owner_id) {
        Entry::Occupied(entry) => entry.into_mut(),
        Entry::Vacant(entry) => {
            let notified = STATE.database.owner_notification(guild_id, owner_id).await?;
            entry.insert(notified)
        }
    };
    if notified.allow_dms == Some(false) || notified.latest_msg_index >= 1 {
        return Ok(());
    }

    let messages = update_messages(crate::interaction::owner_update())?;
    match send_update(owner_id, &messages).await {
        Ok(()) => {
            let notified = OwnerNotified { allow_dms: Some(true), latest_msg_index: 1 };
            STATE.database.save_owner_notification(owner_id, &notified).await?;
            notifications.insert(owner_id, notified);
            tracing::info!(%guild_id, %owner_id, "owner notification delivered");
        }
        Err(ClientError::Http(error))
            if matches!(error.kind(), ErrorType::Response {
                error: ApiError::General(GeneralApiError { code: 50007, .. }),
                ..
            }) =>
        {
            tracing::debug!(%guild_id, %owner_id, "owner notification rejected: direct messages unavailable");
            let notified = OwnerNotified {
                allow_dms: Some(false),
                latest_msg_index: notified.latest_msg_index
            };
            STATE.database.save_owner_notification(owner_id, &notified).await?;
            notifications.insert(owner_id, notified);
        }
        Err(error) => return Err(error.into())
    }
    Ok(())
}

fn update_messages(
    sections: impl IntoIterator<Item = String>
) -> Result<Vec<Vec<Component>>, OwnerNotificationError> {
    let mut messages = Vec::new();
    let mut components = Vec::new();
    let mut length = 0;
    for section in sections {
        let section = section.trim();
        let section_length = section.encode_utf16().count();
        if section_length > MAX_MESSAGE_TEXT_LENGTH {
            return Err(OwnerNotificationError::SectionTooLong(section_length));
        }
        if length + section_length > MAX_MESSAGE_TEXT_LENGTH ||
            components.len() + 3 > MAX_MESSAGE_COMPONENTS
        {
            components.push(opt_out_button());
            messages.push(std::mem::take(&mut components));
            length = 0;
        }
        components
            .push(Component::TextDisplay(TextDisplay { id: None, content: section.to_owned() }));
        length += section_length;
    }
    if !components.is_empty() {
        components.push(opt_out_button());
        messages.push(components);
    }
    Ok(messages)
}

fn opt_out_button() -> Component {
    Component::ActionRow(ActionRow {
        id: None,
        components: vec![Component::Button(Button {
            id: None,
            custom_id: Some(MessageComponentCustomId::OwnerNotificationsDisable.to_string()),
            disabled: false,
            emoji: None,
            label: Some("Don't send me owner notifications".into()),
            style: ButtonStyle::Secondary,
            url: None,
            sku_id: None
        })]
    })
}

async fn send_update(
    owner_id: twilight_model::id::Id<twilight_model::id::marker::UserMarker>,
    messages: &[Vec<Component>]
) -> Result<(), ClientError> {
    let channel = STATE.client.create_private_channel(owner_id).await?.model().await?;
    for components in messages {
        STATE
            .client
            .create_message(channel.id)
            .flags(MessageFlags::IS_COMPONENTS_V2)
            .components(components)
            .await?;
    }
    Ok(())
}
