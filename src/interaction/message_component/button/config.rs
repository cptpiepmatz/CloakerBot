use std::convert::Infallible;
use std::env::var;
use std::str::FromStr;

use twilight_mention::Mention;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{Button, ButtonStyle};
use twilight_model::channel::message::{Component, MessageFlags, ReactionType};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_util::builder::embed::EmbedBuilder;
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::assets::{COLORS, QUOTES, REPLIES};
use crate::interaction::message_component::custom_id::{
    MessageComponentCustomId, ParseMessageComponentCustomIdError
};
use crate::interaction::message_component::HandleMessageComponentError;
use crate::util::template::Template;
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

#[derive(Debug, Copy, Clone)]
pub enum ConfigUnsetButton {
    DangerZones,
    ImmunityGeneral,
    ImmunityRandom,
    LogChannel,
    Random,
    TargetBots
}

impl ToString for ConfigUnsetButton {
    fn to_string(&self) -> String {
        use ConfigUnsetButton::*;
        use MessageComponentCustomId::*;
        (match self {
            DangerZones => ConfigUnsetDangerZones,
            ImmunityGeneral => ConfigUnsetImmunityGeneral,
            ImmunityRandom => ConfigUnsetImmunityRandom,
            LogChannel => ConfigUnsetLogChannel,
            Random => ConfigUnsetRandom,
            TargetBots => ConfigUnsetTargetBots
        })
        .to_string()
    }
}

impl TryFrom<MessageComponentCustomId> for ConfigUnsetButton {
    type Error = ConfigButtonActionError;

    fn try_from(value: MessageComponentCustomId) -> Result<Self, Self::Error> {
        use ConfigUnsetButton::*;
        use MessageComponentCustomId::*;
        Ok(match value {
            ConfigUnsetDangerZones => DangerZones,
            ConfigUnsetImmunityGeneral => ImmunityGeneral,
            ConfigUnsetImmunityRandom => ImmunityRandom,
            ConfigUnsetLogChannel => LogChannel,
            ConfigUnsetRandom => Random,
            ConfigUnsetTargetBots => TargetBots,
            c_id => return Err(ConfigButtonActionError::UnexpectedCustomId(c_id))
        })
    }
}

pub fn component(variant: ConfigUnsetButton) -> Component {
    Component::Button(Button {
        custom_id: Some(variant.to_string()),
        disabled: false,
        emoji: Some(ReactionType::Unicode {
            name: "🗑️".to_string()
        }),
        label: "unset".to_string().into(),
        style: ButtonStyle::Danger,
        url: None
    })
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
    let guild_id = interaction
        .guild_id
        .ok_or(ConfigButtonActionError::NotInGuild)?;
    let mut guild_configs = STATE.guild_configs.write().await;
    let guild_config = guild_configs.entry(guild_id).or_default();

    let variant = ConfigUnsetButton::try_from(custom_id)?;
    match variant {
        ConfigUnsetButton::DangerZones => guild_config.danger_zones.clear(),
        ConfigUnsetButton::ImmunityGeneral => guild_config.immunity.general.clear(),
        ConfigUnsetButton::ImmunityRandom => guild_config.immunity.random.clear(),
        ConfigUnsetButton::LogChannel => guild_config.log_channel = Default::default(),
        ConfigUnsetButton::Random => guild_config.random = Default::default(),
        ConfigUnsetButton::TargetBots => guild_config.target_bots = Default::default()
    }

    match variant {
        ConfigUnsetButton::DangerZones |
        ConfigUnsetButton::ImmunityGeneral |
        ConfigUnsetButton::ImmunityRandom |
        ConfigUnsetButton::LogChannel => {
            // will be pulled out, when the other variant is deprecated fully
            let reply = match variant {
                ConfigUnsetButton::DangerZones => REPLIES
                    .config
                    .danger_zones
                    .unset
                    .template([&QUOTES.config.danger_zones.no_channels]),
                ConfigUnsetButton::ImmunityGeneral => REPLIES
                    .config
                    .immunity
                    .general
                    .unset
                    .template([&QUOTES.config.immunity.general.no_roles]),
                ConfigUnsetButton::ImmunityRandom => REPLIES
                    .config
                    .immunity
                    .random
                    .unset
                    .template([&QUOTES.config.immunity.random.no_roles]),
                ConfigUnsetButton::LogChannel => REPLIES.config.log_channel.unset.template([
                    // TODO: insert fallback log channel here
                    &"[insert fallback log channel here]",
                    &BOT_USER_ID.mention(),
                    &QUOTES.config.log_channel.no_channel
                ]),
                _ => unreachable!()
            };

            let data = InteractionResponseDataBuilder::new()
                .content(reply)
                .flags(MessageFlags::EPHEMERAL)
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

        // TODO: deprecate this
        _ => {
            let (serious_res, snarky_res) = match variant {
                ConfigUnsetButton::DangerZones |
                ConfigUnsetButton::ImmunityGeneral |
                ConfigUnsetButton::ImmunityRandom |
                ConfigUnsetButton::LogChannel => unreachable!(),
                ConfigUnsetButton::Random => (
                    "Unset config for random takedowns. No random takedowns will occur.",
                    "Well, call me if you need me."
                ),
                ConfigUnsetButton::TargetBots => (
                    "Unset config for targeting bots. Bots can be taken down by default.",
                    "Now MEE6 is no more safe than you."
                )
            };

            let embed = EmbedBuilder::new()
                .color(COLORS.primary as u32)
                .title(serious_res)
                .description(snarky_res)
                .build();
            let data = InteractionResponseDataBuilder::new()
                .embeds([embed])
                .flags(MessageFlags::EPHEMERAL)
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
    }
}
