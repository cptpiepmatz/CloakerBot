use std::ops::Deref;

use twilight_mention::Mention;
use twilight_model::channel::Message;
use twilight_http::api_error::{ApiError, GeneralApiError};
use twilight_http::error::ErrorType;

use crate::data::owner_notifications::OwnerNotified;
use crate::util::ClientError;
use crate::{BOT_USER_ID, STATE, assets};

pub async fn on_message(message: &Message) -> Result<(), ClientError> {
    let Some(guild_id) = message.guild_id else { return Ok(()); };
    if message.author.bot { return Ok(()); }
    let owner_id = match STATE.cache.guild(guild_id) {
        Some(guild) => guild.owner_id(),
        None => return Ok(())
    };
    if message.author.id != owner_id { return Ok(()); }

    let _lock = STATE.owner_notification_locks.lock(owner_id).await;
    let notified = STATE.owner_notifications.read().await.acc_get(&owner_id).cloned().unwrap_or_default();
    STATE.owner_notifications.write().await.insert(guild_id, owner_id, notified.clone());
    if notified.allow_dms == Some(false) || notified.latest_msg_index >= 1 { return Ok(()); }

    let content = crate::interaction::owner_update();
    match send_update(owner_id, &content).await {
        Ok(()) => {
            STATE.owner_notifications.write().await.insert(guild_id, owner_id, OwnerNotified {
                allow_dms: Some(true), latest_msg_index: 1
            });
        }
        Err(ClientError::Http(error)) if matches!(error.kind(), ErrorType::Response {
            error: ApiError::General(GeneralApiError { code: 50007, .. }), ..
        }) => {
            STATE.owner_notifications.write().await.insert(guild_id, owner_id, OwnerNotified {
                allow_dms: Some(false), ..notified
            });
        }
        Err(error) => return Err(error)
    }
    Ok(())
}

async fn send_update(owner_id: twilight_model::id::Id<twilight_model::id::marker::UserMarker>, content: &str) -> Result<(), ClientError> {
    let channel = STATE.client.create_private_channel(owner_id).await?.model().await?;
    let mut chunk = String::new();
    let mut length = 0;
    for character in content.chars() {
        if length + character.len_utf16() > 1900 {
            STATE.client.create_message(channel.id).content(&chunk).await?;
            chunk.clear();
            length = 0;
        }
        chunk.push(character);
        length += character.len_utf16();
    }
    if !chunk.is_empty() {
        STATE.client.create_message(channel.id).content(&chunk).await?;
    }
    Ok(())
}
