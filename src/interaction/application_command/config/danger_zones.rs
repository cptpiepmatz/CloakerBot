use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use twilight_mention::Mention;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::CommandDataOption;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::{ButtonStyle, ComponentType};
use twilight_model::channel::ChannelType;
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;
use twilight_util::builder::command::SubCommandBuilder;

use crate::db::config::GuildConfig;
use crate::interaction::application_command::TryFromInteractionError;
use crate::interaction::message_component;
use crate::{State, APPLICATION_ID, STATE};

pub const NAME: &str = "danger-zones";
const DESCRIPTION: &str = "Define your danger zones.";

pub struct DangerZonesSubCommand;

impl TryFrom<&Vec<CommandDataOption>> for DangerZonesSubCommand {
    type Error = TryFromInteractionError;

    fn try_from(options: &Vec<CommandDataOption>) -> Result<Self, Self::Error> {
        if let Some(option) = options.iter().next() {
            return Err(TryFromInteractionError::UnknownField {
                command: format!("{} {}", super::NAME, NAME),
                field: String::from(&option.name)
            });
        }
        Ok(DangerZonesSubCommand)
    }
}

pub fn chat_input() -> CommandOption {
    SubCommandBuilder::new(NAME, DESCRIPTION).build()
}

pub async fn action(
    interaction: Interaction,
    guild_id: Id<GuildMarker>,
    _: DangerZonesSubCommand
) -> anyhow::Result<()> {
    let config = GuildConfig::fetch(guild_id).await?;

    let content = match config.danger_zones.is_empty() {
        true => "**🛠️ *No* danger zones are currently set.**".to_owned(),
        false => format!(
            "**🛠️ The following channels are set as danger zones:** {}",
            config
                .danger_zones
                .iter()
                .map(|c| c.mention().to_string() + "\n")
                .collect::<String>()
        )
    };

    let payload = json!({
        "content": content,
        "components": [
            {
                "type": ComponentType::ActionRow,
                "components": [{
                    // select menu for channel select
                    "type": 8,
                    "custom_id": format!(
                        "{}#{}",
                        message_component::config_danger_zones::SELECT_CUSTOM_ID,
                        interaction.id
                    ),
                    "channel_types": [ChannelType::GuildVoice],
                    "min_values": 0,
                    "max_values": 25,
                    "placeholder": "⚠️ Select channels as danger zones."
                }]
            },
            {
                "type": ComponentType::ActionRow,
                "components": [{
                    // button to remove all channels
                    "type": ComponentType::Button,
                    "custom_id": format!(
                        "{}#{}",
                        message_component::config_danger_zones::BUTTON_CUSTOM_ID,
                        interaction.id
                    ),
                    "label": "Delete Danger Zones",
                    "style": ButtonStyle::Danger,
                    "emoji": {
                        "id": null,
                        "name": "🗑️"
                    }
                }]
            }
        ]
    });

    STATE
        .http
        .interaction(APPLICATION_ID)
        .update_response(&interaction.token)
        .payload_json(payload.to_string().as_bytes())
        .await?;

    let interaction_id = interaction.id;
    STATE
        .tracked_interactions
        .insert(interaction.id, interaction);

    tokio::time::sleep(Duration::from_secs(3 * 60)).await;

    if let Some((_, interaction)) = STATE.tracked_interactions.remove(&interaction_id) {
        let payload = json!({
            "content": format!(
                "**⌛ Timed Out.**\nTry `/{} {}` again to modify danger zones.",
                super::NAME,
                NAME
            ),
            "components": []
        });

        STATE
            .http
            .interaction(APPLICATION_ID)
            .update_response(&interaction.token)
            .payload_json(payload.to_string().as_bytes())
            .await?;
    }

    Ok(())
}
