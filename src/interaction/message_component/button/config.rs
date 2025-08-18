use std::convert::Infallible;
use std::env::var;
use std::ops::Deref;
use std::str::FromStr;

use assets::replies::config;
use derive_more::From;
use twilight_mention::Mention;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::{Component, EmojiReactionType, MessageFlags, ReactionType};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::embed::EmbedBuilder;

use crate::assets::{self, COLORS, QUOTES};
use crate::data::config::{CompleteDefault, GuildConfig};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

#[derive(Debug, Copy, Clone, From)]
pub enum ConfigButton {
    Unset(ConfigUnsetButton),
    Random(ConfigRandomButton)
}

#[derive(Debug, Copy, Clone)]
pub enum ConfigUnsetButton {
    DangerZones,
    ImmunityGeneral,
    ImmunityRandom,
    ImmunityGeneralSilent,
    ImmunityRandomSilent,
    LogChannel
}

#[derive(Debug, Copy, Clone)]
pub enum ConfigRandomButton {
    Enable,
    Disable
}

impl From<ConfigButton> for MessageComponentCustomId {
    fn from(value: ConfigButton) -> Self {
        match value {
            ConfigButton::Unset(cub) => cub.into(),
            ConfigButton::Random(crb) => crb.into()
        }
    }
}

impl From<ConfigUnsetButton> for MessageComponentCustomId {
    fn from(value: ConfigUnsetButton) -> Self {
        use {ConfigUnsetButton as CUB, MessageComponentCustomId as MCCId};

        match value {
            CUB::DangerZones => MCCId::ConfigUnsetDangerZones,
            CUB::ImmunityGeneral => MCCId::ConfigUnsetImmunityGeneral,
            CUB::ImmunityRandom => MCCId::ConfigUnsetImmunityRandom,
            CUB::ImmunityGeneralSilent => MCCId::ConfigUnsetImmunityGeneralSilent,
            CUB::ImmunityRandomSilent => MCCId::ConfigUnsetImmunityRandomSilent,
            CUB::LogChannel => MCCId::ConfigUnsetLogChannel
        }
    }
}

impl From<ConfigRandomButton> for MessageComponentCustomId {
    fn from(value: ConfigRandomButton) -> Self {
        use {ConfigRandomButton as CRB, MessageComponentCustomId as MCCId};

        match value {
            CRB::Enable => MCCId::ConfigEnableRandom,
            CRB::Disable => MCCId::ConfigDisableRandom
        }
    }
}

impl TryFrom<MessageComponentCustomId> for ConfigButton {
    type Error = ConfigButtonActionError;

    fn try_from(value: MessageComponentCustomId) -> Result<Self, Self::Error> {
        use {
            ConfigButton as CB, ConfigRandomButton as CRB, ConfigUnsetButton as CUB,
            MessageComponentCustomId as MCCId
        };

        Ok(match value {
            MCCId::ConfigUnsetDangerZones => CB::Unset(CUB::DangerZones),
            MCCId::ConfigUnsetImmunityGeneral => CB::Unset(CUB::ImmunityGeneral),
            MCCId::ConfigUnsetImmunityRandom => CB::Unset(CUB::ImmunityRandom),
            MCCId::ConfigUnsetImmunityGeneralSilent => CB::Unset(CUB::ImmunityGeneralSilent),
            MCCId::ConfigUnsetImmunityRandomSilent => CB::Unset(CUB::ImmunityRandomSilent),
            MCCId::ConfigUnsetLogChannel => CB::Unset(CUB::LogChannel),
            MCCId::ConfigEnableRandom => CB::Random(CRB::Enable),
            MCCId::ConfigDisableRandom => CB::Random(CRB::Disable),
            c_id => return Err(ConfigButtonActionError::UnexpectedCustomId(c_id))
        })
    }
}

impl ConfigUnsetButton {
    pub fn component(self) -> Component {
        let custom_id = MessageComponentCustomId::from(self).to_string();
        Component::Button(Button {
            custom_id: Some(custom_id),
            disabled: false,
            emoji: Some(EmojiReactionType::Unicode { name: "🗑️".to_string() }),
            label: "unset".to_string().into(),
            style: ButtonStyle::Danger,
            url: None,
            sku_id: None
        })
    }
}

impl ConfigRandomButton {
    pub fn component(self) -> Component {
        let custom_id = MessageComponentCustomId::from(self).to_string();
        Component::Button(match self {
            ConfigRandomButton::Enable => Button {
                custom_id: Some(custom_id),
                disabled: false,
                emoji: Some(EmojiReactionType::Unicode { name: "💫".to_string() }),
                label: Some("enable".to_string()),
                style: ButtonStyle::Primary,
                url: None,
                sku_id: None
            },
            ConfigRandomButton::Disable => Button {
                custom_id: Some(custom_id),
                disabled: false,
                emoji: Some(EmojiReactionType::Unicode { name: "🛑".to_string() }),
                label: Some("disable".to_string()),
                style: ButtonStyle::Danger,
                url: None,
                sku_id: None
            }
        })
    }
}

#[derive(Debug)]
pub enum ConfigButtonActionError {
    NotInGuild,
    UnexpectedCustomId(MessageComponentCustomId)
}

pub async fn action(
    interaction: &Interaction,
    custom_id: MessageComponentCustomId
) -> Result<(), HandleMessageComponentError> {
    use ConfigButton as CB;

    let guild_id = interaction.guild_id.ok_or(ConfigButtonActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();

    let variant = ConfigButton::try_from(custom_id)?;
    let (reply, component) = match variant {
        CB::Unset(cub) => handle_unset(guild_config, cub).await?,
        CB::Random(crb) => handle_random(guild_config, crb).await?
    };

    let data = InteractionResponseDataBuilder::new()
        .content(reply)
        .flags(MessageFlags::EPHEMERAL)
        .components(component)
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

async fn handle_unset(
    guild_config: &mut GuildConfig,
    variant: ConfigUnsetButton
) -> Result<(String, Option<Component>), HandleMessageComponentError> {
    use ConfigUnsetButton as CUB;

    match variant {
        CUB::DangerZones => guild_config.danger_zones.clear(),
        CUB::ImmunityGeneral => guild_config.immunity.general.clear(),
        CUB::ImmunityRandom => guild_config.immunity.random.clear(),
        CUB::ImmunityGeneralSilent => guild_config.immunity.general_silent.clear(),
        CUB::ImmunityRandomSilent => guild_config.immunity.random_silent.clear(),
        CUB::LogChannel => guild_config.log_channel = Default::default()
    }

    // will be pulled out, when the other variant is deprecated fully
    let reply = match variant {
        CUB::DangerZones => config::danger_zones::unset(QUOTES.config.danger_zones.no_channels),
        CUB::ImmunityGeneral => {
            config::immunity::general::unset(QUOTES.config.immunity.general.no_roles)
        }
        CUB::ImmunityRandom => {
            config::immunity::random::unset(QUOTES.config.immunity.random.no_roles)
        }
        CUB::ImmunityGeneralSilent => {
            config::immunity::general::silent::unset(QUOTES.config.immunity.general.no_roles)
        }
        CUB::ImmunityRandomSilent => {
            config::immunity::random::silent::unset(QUOTES.config.immunity.random.no_roles)
        }
        CUB::LogChannel => config::log_channel::unset(
            BOT_USER_ID.mention(),
            // TODO: insert fallback log channel here
            "[insert fallback log channel here]",
            QUOTES.config.log_channel.no_channel
        )
    };

    Ok((reply.into(), None))
}

async fn handle_random(
    guild_config: &mut GuildConfig,
    variant: ConfigRandomButton
) -> Result<(String, Option<Component>), HandleMessageComponentError> {
    use ConfigRandomButton as CRB;

    match variant {
        CRB::Enable => guild_config.random.enabled = Some(true),
        CRB::Disable => guild_config.random.enabled = Some(false)
    }

    let random_config = guild_config.random.complete();

    // FILLME: get a quote here
    let quote = "some quote";
    let reply = match random_config.enabled {
        true => config::random::enabled(
            crate::interaction::application_command::config::random::MENTION.deref(),
            random_config.max,
            random_config.min,
            quote
        ),
        false => config::random::disabled(quote)
    };

    let component = Component::ActionRow(ActionRow {
        components: vec![match variant {
            // use the inverse component to toggle config
            CRB::Enable => CRB::Disable.component(),
            CRB::Disable => CRB::Enable.component()
        }]
    });

    Ok((reply.into(), component.into()))
}
