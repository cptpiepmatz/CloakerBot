use std::env::var;
use std::num::ParseIntError;

use derive_more::From;
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
use twilight_model::id::marker::{ChannelMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::{COLORS, QUOTES, REPLIES};
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::util::template::Template;
use crate::{assets, APPLICATION_ID, STATE};

const CLOAKER_QUOTE_ROLES: &str =
    "It's like they're wrapped in bubble wrap. Oh, how I want to pop those bubbles.";
const CLOAKER_QUOTE_NO_ROLES: &str =
    "Welcome back to the arena. Your immunity vacation is officially over.";

#[derive(Debug, Copy, Clone)]
pub enum ImmunitySelect {
    General,
    Random
}

impl ToString for ImmunitySelect {
    fn to_string(&self) -> String {
        use ImmunitySelect::*;
        use MessageComponentCustomId::*;
        (match self {
            General => ImmunityGeneralSelect,
            Random => ImmunityRandomSelect
        })
        .to_string()
    }
}

impl TryFrom<MessageComponentCustomId> for ImmunitySelect {
    type Error = ImmunitySelectActionError;

    fn try_from(value: MessageComponentCustomId) -> Result<Self, Self::Error> {
        use ImmunitySelect::*;
        use MessageComponentCustomId::*;
        Ok(match value {
            ImmunityGeneralSelect => General,
            ImmunityRandomSelect => Random,
            c_id => return Err(ImmunitySelectActionError::UnexpectedCustomId(c_id))
        })
    }
}

pub fn component(variant: ImmunitySelect) -> Value {
    // TODO: update this when https://github.com/twilight-rs/twilight/issues/2047 gets resolved
    json!({
        // select menu for channel select
        "type": 6,
        "custom_id": variant.to_string(),
        "min_values": 0,
        "max_values": 25,
        "placeholder": "🛡️ Select roles to be immune."
    })
}

#[derive(Debug, From)]
pub enum ImmunitySelectActionError {
    NotInGuild,
    UnexpectedCustomId(MessageComponentCustomId),
    InvalidValues(Vec<String>)
}

pub async fn action(
    interaction: &Interaction,
    custom_id: MessageComponentCustomId,
    data: &MessageComponentInteractionData
) -> Result<(), HandleMessageComponentError> {
    let guild_id = interaction
        .guild_id
        .ok_or(ImmunitySelectActionError::NotInGuild)?;

    let variant = ImmunitySelect::try_from(custom_id)?;

    type RoleId = Id<RoleMarker>;
    let roles: Result<Vec<RoleId>, _> = data.values.iter().map(|v| v.parse::<RoleId>()).collect();
    let roles = roles.map_err(|_| ImmunitySelectActionError::InvalidValues(data.values.clone()))?;

    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    match variant {
        ImmunitySelect::General => guild_config.immunity.general = roles.clone(),
        ImmunitySelect::Random => guild_config.immunity.random = roles.clone()
    }

    let self_mention = APPLICATION_ID.cast::<UserMarker>().mention();
    let roles_insert = roles
        .iter()
        .map(|r| format!("- {}", r.mention()))
        .join("\n");
    let content = match (roles.is_empty(), variant) {
        (true, ImmunitySelect::General) => REPLIES
            .config
            .immunity
            .general
            .unset
            .template([&QUOTES.config.immunity.general.no_roles]),
        (true, ImmunitySelect::Random) => REPLIES.config.immunity.random.unset.template([&QUOTES
            .config
            .immunity
            .random
            .no_roles]),
        (false, ImmunitySelect::General) => REPLIES
            .config
            .immunity
            .general
            .set
            .template([&self_mention, &roles_insert, &QUOTES.config.immunity.general.roles]),
        (false, ImmunitySelect::Random) => REPLIES
            .config
            .immunity
            .random
            .set
            .template([&self_mention, &roles_insert, &QUOTES.config.immunity.random.roles])
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
