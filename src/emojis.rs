use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use twilight_model::guild::Emoji;
use twilight_model::id::Id;
use twilight_model::id::marker::EmojiMarker;

use crate::assets::graphics::user_rank::badges::*;
use crate::panic::ExpectReport;
use crate::{APPLICATION_ID, STATE};

/// Application emoji IDs, usable in message text with
/// `twilight_mention::Mention`.
#[derive(Debug, Clone, Copy)]
pub struct EmojisRegister {
    pub immunity: Id<EmojiMarker>,
    pub never_initiated: Id<EmojiMarker>,
    pub never_taken_down: Id<EmojiMarker>,
    pub random_immunity: Id<EmojiMarker>,
    pub top_counterer: Id<EmojiMarker>,
    pub top_striker: Id<EmojiMarker>,
    pub top_victim: Id<EmojiMarker>
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterEmojisError {
    #[error("could not register application emojis with Discord")]
    Http(#[from] twilight_http::Error),

    #[error("could not decode application emojis")]
    DeserializeBody(#[from] twilight_http::response::DeserializeBodyError)
}

pub static EMOJIS_REGISTER: LazyLock<EmojisRegister> = LazyLock::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(register())
            .expect_report("could not register application emojis")
    })
});

#[tracing::instrument(skip_all)]
async fn register() -> Result<EmojisRegister, RegisterEmojisError> {
    let emojis = STATE.client.get_application_emojis(APPLICATION_ID).await?.model().await?.items;

    Ok(EmojisRegister {
        immunity: ensure_emoji(&emojis, "immunity", IMMUNITY).await?,
        never_initiated: ensure_emoji(&emojis, "never_initiated", NEVER_INITIATED).await?,
        never_taken_down: ensure_emoji(&emojis, "never_taken_down", NEVER_TAKEN_DOWN).await?,
        random_immunity: ensure_emoji(&emojis, "random_immunity", RANDOM_IMMUNITY).await?,
        top_counterer: ensure_emoji(&emojis, "top_counterer", TOP_COUNTERER).await?,
        top_striker: ensure_emoji(&emojis, "top_striker", TOP_STRIKER).await?,
        top_victim: ensure_emoji(&emojis, "top_victim", TOP_VICTIM).await?
    })
}

async fn ensure_emoji(
    emojis: &[Emoji],
    name: &str,
    image: &[u8]
) -> Result<Id<EmojiMarker>, RegisterEmojisError> {
    if let Some(emoji) = emojis.iter().find(|emoji| emoji.name == name) {
        tracing::trace!(name, emoji_id = %emoji.id, "found application emoji");
        return Ok(emoji.id);
    }

    let image = format!("data:image/webp;base64,{}", STANDARD.encode(image));
    let emoji =
        STATE.client.add_application_emoji(APPLICATION_ID, name, &image).await?.model().await?;
    tracing::info!(name, emoji_id = %emoji.id, "created application emoji");

    Ok(emoji.id)
}
