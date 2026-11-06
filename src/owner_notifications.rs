use std::collections::hash_map::Entry;

use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_http::error::ErrorType;
use twilight_mention::Mention;
use twilight_model::channel::Message;
use twilight_model::channel::message::MessageFlags;
use twilight_model::channel::message::component::Component;

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
    ComponentTooLong(usize),
    #[from(skip)]
    TooManyComponents(usize)
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
            let notified = STATE.database.owner_notification(owner_id).await?;
            entry.insert(notified)
        }
    };
    if notified.allow_dms == Some(false) || notified.latest_msg_index >= 1 {
        return Ok(());
    }

    let messages = update_messages(owner_update())?;
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

fn owner_update() -> impl Iterator<Item = impl Into<Component>> {
    crate::assets::owner_notifications::v3_update(
        crate::BOT_USER_ID.mention(),
        MessageComponentCustomId::OwnerNotificationsDisable.to_string()
    )
}

fn update_messages(
    templates: impl IntoIterator<Item = impl Into<Component>>
) -> Result<Vec<Vec<Component>>, OwnerNotificationError> {
    let mut messages = Vec::new();
    let mut components = Vec::new();
    let mut component_count = 0;
    let mut length = 0;
    for template in templates {
        let component = template.into();
        let (count, text_length) = component_metrics(&component);
        if text_length > MAX_MESSAGE_TEXT_LENGTH {
            return Err(OwnerNotificationError::ComponentTooLong(text_length));
        }
        if count > MAX_MESSAGE_COMPONENTS {
            return Err(OwnerNotificationError::TooManyComponents(count));
        }
        if length + text_length > MAX_MESSAGE_TEXT_LENGTH ||
            component_count + count > MAX_MESSAGE_COMPONENTS
        {
            messages.push(std::mem::take(&mut components));
            length = 0;
            component_count = 0;
        }
        components.push(component);
        length += text_length;
        component_count += count;
    }
    if !components.is_empty() {
        messages.push(components);
    }
    Ok(messages)
}

fn component_metrics(component: &Component) -> (usize, usize) {
    let children = match component {
        Component::TextDisplay(text) => return (1, text.content.encode_utf16().count()),
        Component::Container(container) => &container.components,
        Component::ActionRow(row) => &row.components,
        _ => return (1, 0)
    };
    children
        .iter()
        .map(component_metrics)
        .fold((1, 0), |(count, length), (child_count, child_length)| {
            (count + child_count, length + child_length)
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
