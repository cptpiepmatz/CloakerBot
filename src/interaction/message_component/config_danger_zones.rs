use std::str::FromStr;
use std::sync::Arc;

use serde_json::json;
use twilight_mention::Mention;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::ComponentType;
use twilight_model::id::marker::{ChannelMarker, InteractionMarker};
use twilight_model::id::Id;

use crate::db::config::GuildConfig;
use crate::interaction::message_component::TryFromInteractionError;
use crate::{db, State, APPLICATION_ID, STATE};

pub const SELECT_CUSTOM_ID: &str = "cfg_dz_select";
pub const BUTTON_CUSTOM_ID: &str = "cfg_dz_button";

pub struct ConfigDangerZonesSelect {
    pub og_interaction_id: Id<InteractionMarker>,
    pub channels: Vec<Id<ChannelMarker>>
}

pub struct ConfigDangerZonesButton {
    pub og_interaction_id: Id<InteractionMarker>
}

impl TryFrom<(&MessageComponentInteractionData, &str)> for ConfigDangerZonesSelect {
    type Error = TryFromInteractionError;

    fn try_from(
        (data, og_id): (&MessageComponentInteractionData, &str)
    ) -> Result<Self, Self::Error> {
        match data.component_type {
            ComponentType::Unknown(8) => {}
            kind => {
                return Err(TryFromInteractionError::IncorrectType {
                    custom_id: SELECT_CUSTOM_ID.to_owned(),
                    expected: ComponentType::Unknown(8),
                    actual: kind
                })
            }
        }

        Ok(ConfigDangerZonesSelect {
            og_interaction_id: Id::from_str(og_id).map_err(|_| {
                TryFromInteractionError::ParseId {
                    value: og_id.to_owned()
                }
            })?,
            channels: data
                .values
                .iter()
                .map(|id| Id::from_str(id).expect("validated by Discord"))
                .collect()
        })
    }
}

impl TryFrom<(&MessageComponentInteractionData, &str)> for ConfigDangerZonesButton {
    type Error = TryFromInteractionError;

    fn try_from(
        (data, og_id): (&MessageComponentInteractionData, &str)
    ) -> Result<Self, Self::Error> {
        match data.component_type {
            ComponentType::Button => {}
            kind => {
                return Err(TryFromInteractionError::IncorrectType {
                    custom_id: BUTTON_CUSTOM_ID.to_owned(),
                    expected: ComponentType::Button,
                    actual: kind
                })
            }
        }

        Ok(ConfigDangerZonesButton {
            og_interaction_id: Id::from_str(og_id).map_err(|_| {
                TryFromInteractionError::ParseId {
                    value: og_id.to_owned()
                }
            })?
        })
    }
}

pub async fn action(
    interaction: Interaction,
    og_interaction_id: Id<InteractionMarker>,
    channels: Vec<Id<ChannelMarker>>
) -> anyhow::Result<()> {
    let og_interaction = match STATE.tracked_interactions.remove(&og_interaction_id) {
        Some((_, interaction)) => interaction,
        None => return Ok(())
    };

    let Some(guild_id) = interaction.guild_id else {
        // ignore if not in guild
        return Ok(())
    };

    let mut config = GuildConfig::fetch(guild_id).await?;

    let reply = match channels.is_empty() {
        true => "**🛠️ All danger zones *removed*.**".to_string(),
        false => format!(
            "**🛠️ The following channels are now danger zones:**\n{}",
            channels
                .iter()
                .map(|c| c.mention().to_string() + "\n")
                .collect::<String>()
        )
    };

    config.danger_zones = channels;
    config.save(guild_id).await?;

    let payload = json!({
        "content": reply,
        "components": []
    });

    STATE
        .http
        .interaction(APPLICATION_ID)
        .update_response(&og_interaction.token)
        .payload_json(payload.to_string().as_bytes())
        .await?;

    Ok(())
}
