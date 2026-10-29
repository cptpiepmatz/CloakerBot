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
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::embed::EmbedBuilder;

use crate::assets::{self, QUOTES};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::interaction::message_component::channel_select::danger_zones::DangerZonesSelectActionError;
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub fn component(current: Option<Id<ChannelMarker>>) -> Component {
    let default_values: Vec<_> =
        current.into_iter().map(|channel_id| SelectDefaultValue::Channel(channel_id)).collect();

    Component::SelectMenu(SelectMenu {
        channel_types: Some(vec![ChannelType::GuildText]),
        custom_id: MessageComponentCustomId::LogChannelSelect.to_string(),
        default_values: Some(default_values),
        disabled: false,
        kind: SelectMenuType::Channel,
        max_values: Some(1),
        min_values: Some(0),
        options: None,
        placeholder: Some("📝 Select a channel for logging.".to_string())
    })
}

#[derive(Debug)]
pub enum LogChannelSelectActionError {
    NotInGuild,
    InvalidValue(String)
}

pub async fn action(
    interaction: &Interaction,
    data: &MessageComponentInteractionData
) -> Result<(), HandleMessageComponentError> {
    let guild_id = interaction.guild_id.ok_or(LogChannelSelectActionError::NotInGuild)?;

    type ChId = Id<ChannelMarker>;
    let channel: Option<ChId> = if let Some(raw_id) = data.values.iter().next() {
        match raw_id.parse::<ChId>() {
            Ok(id) => Some(id),
            Err(e) => return Err(LogChannelSelectActionError::InvalidValue(e.to_string()).into())
        }
    }
    else {
        None
    };

    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    guild_config.log_channel = channel;
    drop(guild_configs);

    let content = match channel {
        Some(ch_id) => assets::replies::config::log_channel::set(
            BOT_USER_ID.mention(),
            ch_id.mention(),
            QUOTES.config.log_channel.channel
        ),
        None => assets::replies::config::log_channel::unset(
            BOT_USER_ID.mention(),
            crate::logging::fallback_description(guild_id).await,
            QUOTES.config.log_channel.no_channel
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
