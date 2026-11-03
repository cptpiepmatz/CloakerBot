use std::sync::LazyLock;

use thiserror::Error;
use twilight_mention::Mention;
use twilight_mention::fmt::{CommandMention, MentionFormat};
use twilight_model::application::command::{Command, CommandType};
use twilight_model::application::interaction::{Interaction, InteractionContextType};
use twilight_model::channel::message::component::{
    Button, ButtonStyle, Section, Separator, SeparatorSpacingSize, TextDisplay
};
use twilight_model::channel::message::{Component, MessageFlags};
use twilight_model::guild::Permissions;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};
use twilight_model::id::Id;
use twilight_model::id::marker::CommandMarker;
use twilight_util::builder::InteractionResponseDataBuilder;
use twilight_util::builder::command::CommandBuilder;

use super::{COMMANDS_REGISTER, HandleApplicationCommandError};
use crate::interaction::message_component::custom_id::MessageComponentCustomId;
use crate::{APPLICATION_ID, BOT_USER_ID, STATE};

pub const NAME: &str = "config";
pub const DESCRIPTION: &str = "Configure the CloakerBot.";
pub static ID: LazyLock<Id<CommandMarker>> = LazyLock::new(|| COMMANDS_REGISTER.config);
pub static MENTION: LazyLock<MentionFormat<CommandMention>> = super::mention![command];

pub fn chat_input() -> Command {
    CommandBuilder::new(NAME, DESCRIPTION, CommandType::ChatInput)
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .contexts([InteractionContextType::Guild])
        .build()
}

#[derive(Debug, Error)]
pub enum ConfigActionError {
    #[error("not in guild")]
    NotInGuild
}

fn section(title: &str, description: &str, custom_id: MessageComponentCustomId) -> Component {
    Component::Section(Section {
        id: None,
        components: vec![Component::TextDisplay(TextDisplay {
            id: None,
            content: format!("### {title}\n{description}")
        })],
        accessory: Box::new(Component::Button(Button {
            id: None,
            custom_id: Some(custom_id.to_string()),
            disabled: false,
            emoji: None,
            label: Some("Configure".into()),
            style: ButtonStyle::Primary,
            url: None,
            sku_id: None
        }))
    })
}

pub async fn action(interaction: &Interaction) -> Result<(), HandleApplicationCommandError> {
    interaction.guild_id.ok_or(ConfigActionError::NotInGuild)?;
    use MessageComponentCustomId as Id;
    let data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL | MessageFlags::IS_COMPONENTS_V2)
        .components([
            Component::TextDisplay(TextDisplay {
                id: None,
                content: crate::assets::replies::config::menu(BOT_USER_ID.mention()).to_string()
            }),
            Component::Separator(Separator {
                id: None,
                divider: Some(true),
                spacing: Some(SeparatorSpacingSize::Large)
            }),
            section(
                "🪤 Danger Zones",
                "Choose voice channels that trigger a takedown when joined.",
                Id::ConfigOpenDangerZones
            ),
            Component::Separator(Separator {
                id: None,
                divider: Some(true),
                spacing: Some(SeparatorSpacingSize::Large)
            }),
            section(
                "📝 Log Channel",
                "Choose where takedown logs and updates are posted.",
                Id::ConfigOpenLogChannel
            ),
            Component::Separator(Separator {
                id: None,
                divider: Some(true),
                spacing: Some(SeparatorSpacingSize::Large)
            }),
            section(
                "🛡️ Immunity",
                "Choose roles protected from general and random takedowns, including silent \
                 immunity.",
                Id::ConfigOpenImmunity
            ),
            Component::Separator(Separator {
                id: None,
                divider: Some(true),
                spacing: Some(SeparatorSpacingSize::Large)
            }),
            section(
                "💫 Random Takedowns",
                "Enable or disable random takedowns and set their timing.",
                Id::ConfigOpenRandom
            )
        ])
        .build();
    STATE
        .client
        .interaction(APPLICATION_ID)
        .create_response(interaction.id, &interaction.token, &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data)
        })
        .await?;
    Ok(())
}
