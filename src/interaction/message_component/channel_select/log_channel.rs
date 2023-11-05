use std::num::ParseIntError;

use itertools::Itertools;
use serde_json::{json, Value};
use twilight_mention::Mention;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::component::{Button, ButtonStyle};
use twilight_model::channel::message::embed::EmbedField;
use twilight_model::channel::message::{Component, Embed, MessageFlags, ReactionType};
use twilight_model::channel::ChannelType;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::{QUOTES, REPLIES};
use crate::interaction::message_component::channel_select::danger_zones::DangerZonesSelectActionError;
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::util::template::Template;
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub fn component(current: Option<Id<ChannelMarker>>) -> Value {
    let default_values: Vec<_> = current
        .iter()
        .map(|channel_id| {
            json!({
                "id": channel_id,
                "type": "channel"
            })
        })
        .collect();

    // TODO: update this when https://github.com/twilight-rs/twilight/issues/2047 gets resolved
    json!({
        // select menu for channel select
        "type": 8,
        "custom_id": MessageComponentCustomId::LogChannelSelect,
        "channel_types": [ChannelType::GuildText],
        "min_values": 0,
        "max_values": 1,
        "default_values": default_values,
        "placeholder": "📝 Select a channel for logging."
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
    let guild_id = interaction
        .guild_id
        .ok_or(LogChannelSelectActionError::NotInGuild)?;

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

    let content = match channel {
        Some(ch_id) => REPLIES.config.log_channel.set.template([
            &ch_id.mention(),
            &BOT_USER_ID.mention(),
            &QUOTES.config.log_channel.channel
        ]),
        None => REPLIES.config.log_channel.unset.template([
            // TODO: insert fallback log channel here
            &"[insert fallback here]",
            &APPLICATION_ID.cast::<UserMarker>().mention(),
            &QUOTES.config.log_channel.no_channel
        ])
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
