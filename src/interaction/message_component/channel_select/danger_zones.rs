use std::num::ParseIntError;

use serde_json::{json, Value};
use twilight_mention::Mention;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::component::{Button, ButtonStyle};
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::{Component, Embed, MessageFlags, ReactionType};
use twilight_model::channel::ChannelType;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::util::impl_from_variant::impl_from_variant;
use crate::{assets, APPLICATION_ID, STATE};

const CLOAKER_QUOTE_CHANNELS: &str =
    "Welcome to the arena of your demise, where pain becomes your only companion.";
const CLOAKER_QUOTE_NO_CHANNELS: &str = "No longer my turf, but don’t mistake absence for safety.";

pub fn component() -> Value {
    // TODO: update this when https://github.com/twilight-rs/twilight/issues/2047 gets resolved
    json!({
        // select menu for channel select
        "type": 8,
        "custom_id": MessageComponentCustomId::DangerZonesSelect,
        "channel_types": [ChannelType::GuildVoice],
        "min_values": 0,
        "max_values": 25,
        "placeholder": "⚠️ Select channels as danger zones."
    })
}

#[derive(Debug)]
pub enum DangerZonesSelectActionError {
    NotInGuild,
    InvalidValues(Vec<String>)
}

impl_from_variant!(
    DangerZonesSelectActionError,
    HandleMessageComponentError,
    DangerZonesSelectAction
);

pub async fn action(
    interaction: &Interaction,
    data: &MessageComponentInteractionData
) -> Result<(), HandleMessageComponentError> {
    let guild_id = interaction
        .guild_id
        .ok_or(DangerZonesSelectActionError::NotInGuild)?;

    type ChId = Id<ChannelMarker>;
    let channels: Result<Vec<ChId>, _> = data.values.iter().map(|v| v.parse::<ChId>()).collect();
    let channels =
        channels.map_err(|_| DangerZonesSelectActionError::InvalidValues(data.values.clone()))?;

    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    guild_config.danger_zones = channels.clone();

    let content = match channels.is_empty() {
        true => format!(
            "
## 🗑️ Unset Danger Zones
No danger zones are set now.

> {}
            ",
            CLOAKER_QUOTE_NO_CHANNELS
        ),
        false => format!(
            "
## 📌 Configured Danger Zones
Set the following channels as danger zones:
{}

> {}
        ",
            channels
                .iter()
                .map(|c_id| format!("- {}", c_id.mention()))
                .collect::<Vec<String>>()
                .join("\n"),
            CLOAKER_QUOTE_CHANNELS
        )
    };

    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .build();
    STATE
        .http
        .interaction(APPLICATION_ID)
        .create_response(
            interaction.id,
            interaction.token.as_str(),
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(data)
            }
        )
        .await?;

    Ok(())
}
