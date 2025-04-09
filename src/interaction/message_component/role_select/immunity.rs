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
use crate::interaction::message_component::button::config::ConfigUnsetButton::ImmunityGeneralSilent;
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::util::template::Template;
use crate::{assets, APPLICATION_ID, BOT_USER_ID, STATE};

const CLOAKER_QUOTE_ROLES: &str =
    "It's like they're wrapped in bubble wrap. Oh, how I want to pop those bubbles.";
const CLOAKER_QUOTE_NO_ROLES: &str =
    "Welcome back to the arena. Your immunity vacation is officially over.";

#[derive(Debug, Copy, Clone)]
pub enum ImmunitySelect {
    General,
    Random,
    GeneralSilent,
    RandomSilent
}

impl ToString for ImmunitySelect {
    fn to_string(&self) -> String {
        type IS = ImmunitySelect;
        type MCCId = MessageComponentCustomId;
        (match self {
            IS::General => MCCId::ImmunityGeneralSelect,
            IS::Random => MCCId::ImmunityRandomSelect,
            IS::GeneralSilent => MCCId::ImmunityGeneralSilentSelect,
            IS::RandomSilent => MCCId::ImmunityRandomSilentSelect
        })
        .to_string()
    }
}

impl TryFrom<MessageComponentCustomId> for ImmunitySelect {
    type Error = ImmunitySelectActionError;

    fn try_from(value: MessageComponentCustomId) -> Result<Self, Self::Error> {
        type IS = ImmunitySelect;
        type MCCId = MessageComponentCustomId;
        Ok(match value {
            MCCId::ImmunityGeneralSelect => IS::General,
            MCCId::ImmunityRandomSelect => IS::Random,
            MCCId::ImmunityGeneralSilentSelect => IS::GeneralSilent,
            MCCId::ImmunityRandomSilentSelect => IS::RandomSilent,
            c_id => return Err(ImmunitySelectActionError::UnexpectedCustomId(c_id))
        })
    }
}

pub fn component(variant: ImmunitySelect, current: &[Id<RoleMarker>]) -> Value {
    // TODO: update this when https://github.com/twilight-rs/twilight/issues/2047 gets resolved
    let default_values: Vec<_> = current
        .iter()
        .map(|rid| {
            json!({
                "id": rid,
                "type": "role"
            })
        })
        .collect();

    json!({
        // select menu for channel select
        "type": 6,
        "custom_id": variant.to_string(),
        "min_values": 0,
        "max_values": 25,
        "placeholder": "🛡️ Select roles to be immune.",
        "default_values": default_values
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
    let guild_id = interaction.guild_id.ok_or(ImmunitySelectActionError::NotInGuild)?;

    let variant = ImmunitySelect::try_from(custom_id)?;

    type RoleId = Id<RoleMarker>;
    let roles: Result<Vec<RoleId>, _> = data.values.iter().map(|v| v.parse::<RoleId>()).collect();
    let roles = roles.map_err(|_| ImmunitySelectActionError::InvalidValues(data.values.clone()))?;

    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();
    match variant {
        ImmunitySelect::General => guild_config.immunity.general = roles.clone(),
        ImmunitySelect::Random => guild_config.immunity.random = roles.clone(),
        ImmunitySelect::GeneralSilent => guild_config.immunity.general_silent = roles.clone(),
        ImmunitySelect::RandomSilent => guild_config.immunity.random_silent = roles.clone()
    }

    let roles_insert = roles.iter().map(|r| format!("- {}", r.mention())).join("\n");
    let content = match (roles.is_empty(), variant) {
        (true, ImmunitySelect::General) => REPLIES
            .config
            .immunity
            .general
            .unset
            .template([&QUOTES.config.immunity.general.no_roles]),
        (true, ImmunitySelect::Random) => {
            REPLIES.config.immunity.random.unset.template([&QUOTES.config.immunity.random.no_roles])
        }
        (true, ImmunitySelect::GeneralSilent) => {
            REPLIES.config.immunity.general.silent.unset.template([&QUOTES
                .config
                .immunity
                .general
                .no_roles])
        }
        (true, ImmunitySelect::RandomSilent) => {
            REPLIES.config.immunity.random.silent.unset.template([&QUOTES
                .config
                .immunity
                .random
                .no_roles])
        }
        (false, ImmunitySelect::General) => REPLIES.config.immunity.general.set.template([
            &BOT_USER_ID.mention(),
            &roles_insert,
            &QUOTES.config.immunity.general.roles
        ]),
        (false, ImmunitySelect::Random) => REPLIES.config.immunity.random.set.template([
            &BOT_USER_ID.mention(),
            &roles_insert,
            &QUOTES.config.immunity.random.roles
        ]),
        (false, ImmunitySelect::GeneralSilent) => {
            REPLIES.config.immunity.general.silent.set.template([
                &BOT_USER_ID.mention(),
                &roles_insert,
                &QUOTES.config.immunity.general.roles
            ])
        }
        (false, ImmunitySelect::RandomSilent) => {
            REPLIES.config.immunity.random.silent.set.template([
                &BOT_USER_ID.mention(),
                &roles_insert,
                &QUOTES.config.immunity.random.roles
            ])
        }
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
