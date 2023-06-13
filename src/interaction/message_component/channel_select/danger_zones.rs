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

    let embed_builder = EmbedBuilder::new().color(assets::colors::PRIMARY);
    let mut embed: Embed;
    if channels.is_empty() {
        embed = embed_builder
            .title("No channel set as danger zones.")
            .description("Don't think you're safe in there.")
            .build();
    }
    else {
        let mut channels_field_content = String::new();
        for channel in channels {
            channels_field_content.push_str(format!("{}\n", channel.mention()).as_str());
        }
        embed = embed_builder
            .title("Set channels as danger zones.")
            .description("I will have my fun in there!")
            .field(EmbedField {
                inline: false,
                name: "Channels".to_string(),
                value: channels_field_content
            })
            .build();
    }

    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .embeds([embed])
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
