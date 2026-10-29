use std::num::ParseIntError;

use itertools::Itertools;
use serde_json::{Value, json};
use twilight_mention::Mention;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::ChannelType;
use twilight_model::channel::message::component::{
    Button, ButtonStyle, SelectDefaultValue, SelectMenu, SelectMenuType
};
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::{Component, Embed, MessageFlags, ReactionType};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::embed::EmbedBuilder;

use crate::assets::{self, QUOTES};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::{APPLICATION_ID, STATE};

pub fn component(current: &[Id<ChannelMarker>]) -> Component {
    let default_values: Vec<_> =
        current.iter().map(|cid| SelectDefaultValue::Channel(*cid)).collect();

    Component::SelectMenu(SelectMenu {
        id: None,
        required: None,
        channel_types: Some(vec![ChannelType::GuildVoice]),
        custom_id: MessageComponentCustomId::DangerZonesSelect.to_string(),
        default_values: Some(default_values),
        disabled: false,
        kind: SelectMenuType::Channel,
        max_values: Some(25),
        min_values: Some(0),
        options: None,
        placeholder: Some("⚠️ Select channels as danger zones.".to_string())
    })
}

#[derive(Debug)]
pub enum DangerZonesSelectActionError {
    NotInGuild,
    InvalidValues(Vec<String>)
}

pub async fn action(
    interaction: &Interaction,
    data: &MessageComponentInteractionData
) -> Result<(), HandleMessageComponentError> {
    let guild_id = interaction.guild_id.ok_or(DangerZonesSelectActionError::NotInGuild)?;

    type ChId = Id<ChannelMarker>;
    let channels: Result<Vec<ChId>, _> = data.values.iter().map(|v| v.parse::<ChId>()).collect();
    let channels =
        channels.map_err(|_| DangerZonesSelectActionError::InvalidValues(data.values.clone()))?;

    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    guild_config.danger_zones = channels.clone();

    let content = match channels.is_empty() {
        true => {
            assets::replies::config::danger_zones::unset(QUOTES.config.danger_zones.no_channels)
        }
        false => assets::replies::config::danger_zones::set(
            channels.iter().map(|c_id| format!("- {}", c_id.mention())).join("\n"),
            QUOTES.config.danger_zones.channels
        )
    };

    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .content(content)
        .components(None)
        .build();
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, interaction.token.as_str(), &InteractionResponse {
            kind: InteractionResponseType::UpdateMessage,
            data: Some(data)
        })
        .await?;

    Ok(())
}
