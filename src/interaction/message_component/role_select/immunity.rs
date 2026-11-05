use std::env::var;
use std::num::ParseIntError;

use derive_more::From;
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
use twilight_model::id::marker::{ChannelMarker, RoleMarker, UserMarker};
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::embed::EmbedBuilder;

use crate::assets::{COLORS, QUOTES};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::interaction::message_component::button::config::ConfigUnsetButton::ImmunityGeneralSilent;
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::{APPLICATION_ID, BOT_USER_ID, STATE, assets};

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

impl ImmunitySelect {
    pub fn as_str(&self) -> &'static str {
        (*self).into()
    }
}

impl From<ImmunitySelect> for MessageComponentCustomId {
    fn from(value: ImmunitySelect) -> Self {
        type IS = ImmunitySelect;
        type MCCId = MessageComponentCustomId;
        match value {
            IS::General => MCCId::ImmunityGeneralSelect,
            IS::Random => MCCId::ImmunityRandomSelect,
            IS::GeneralSilent => MCCId::ImmunityGeneralSilentSelect,
            IS::RandomSilent => MCCId::ImmunityRandomSilentSelect
        }
    }
}

impl From<ImmunitySelect> for &'static str {
    fn from(value: ImmunitySelect) -> Self {
        MessageComponentCustomId::from(value).into()
    }
}

impl std::fmt::Display for ImmunitySelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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

pub fn component(variant: ImmunitySelect, current: &[Id<RoleMarker>]) -> Component {
    let default_values: Vec<_> = current.iter().map(|rid| SelectDefaultValue::Role(*rid)).collect();

    Component::SelectMenu(SelectMenu {
        id: None,
        required: None,
        channel_types: None,
        custom_id: variant.to_string(),
        default_values: Some(default_values),
        disabled: false,
        kind: SelectMenuType::Role,
        max_values: Some(25),
        min_values: Some(0),
        options: None,
        placeholder: Some("🛡️ Select roles to be immune.".to_string())
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

    STATE
        .database
        .update_config(guild_id, |config| match variant {
            ImmunitySelect::General => config.immunity.general = roles.clone(),
            ImmunitySelect::Random => config.immunity.random = roles.clone(),
            ImmunitySelect::GeneralSilent => config.immunity.general_silent = roles.clone(),
            ImmunitySelect::RandomSilent => config.immunity.random_silent = roles.clone()
        })
        .await?;

    let roles_insert = roles.iter().map(|r| format!("- {}", r.mention())).join("\n");
    use assets::replies::config;
    let content = match (roles.is_empty(), variant) {
        (true, ImmunitySelect::General) => {
            config::immunity::general::unset(QUOTES.config.immunity.general.no_roles).to_string()
        }
        (true, ImmunitySelect::Random) => {
            config::immunity::random::unset(QUOTES.config.immunity.random.no_roles).to_string()
        }
        (true, ImmunitySelect::GeneralSilent) => {
            config::immunity::general::silent::unset(QUOTES.config.immunity.general.no_roles)
                .to_string()
        }
        (true, ImmunitySelect::RandomSilent) => {
            config::immunity::random::silent::unset(QUOTES.config.immunity.random.no_roles)
                .to_string()
        }
        (false, ImmunitySelect::General) => config::immunity::general::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.general.roles,
            roles_insert
        )
        .to_string(),
        (false, ImmunitySelect::Random) => config::immunity::random::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.random.roles,
            roles_insert
        )
        .to_string(),
        (false, ImmunitySelect::GeneralSilent) => config::immunity::general::silent::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.general.roles,
            roles_insert
        )
        .to_string(),
        (false, ImmunitySelect::RandomSilent) => config::immunity::random::silent::set(
            BOT_USER_ID.mention(),
            QUOTES.config.immunity.random.roles,
            roles_insert
        )
        .to_string()
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
