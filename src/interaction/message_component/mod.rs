mod config_danger_zones;

pub mod custom_id {
    pub const CONFIG_DANGER_ZONES_SELECT: &str = super::config_danger_zones::SELECT_CUSTOM_ID;
    pub const CONFIG_DANGER_ZONES_BUTTON: &str = super::config_danger_zones::BUTTON_CUSTOM_ID;
}

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use twilight_model::application::interaction::{Interaction, InteractionData};
use twilight_model::channel::message::component::ComponentType;

use crate::interaction::message_component::config_danger_zones::{
    ConfigDangerZonesButton, ConfigDangerZonesSelect
};
use crate::Com;

enum MessageComponent {
    ConfigDangerZonesSelect(ConfigDangerZonesSelect),
    ConfigDangerZonesButton(ConfigDangerZonesButton)
}

impl TryFrom<&Interaction> for MessageComponent {
    type Error = TryFromInteractionError;

    fn try_from(interaction: &Interaction) -> Result<Self, Self::Error> {
        let Some(InteractionData::MessageComponent(data)) = &interaction.data else {
            return Err(TryFromInteractionError::NoMessageComponent);
        };

        let mut custom_id_split = data.custom_id.split('#');
        match (custom_id_split.next(), custom_id_split.next()) {
            (Some(config_danger_zones::SELECT_CUSTOM_ID), Some(og_id)) => Ok(
                Self::ConfigDangerZonesSelect(ConfigDangerZonesSelect::try_from((data, og_id))?)
            ),
            (Some(config_danger_zones::BUTTON_CUSTOM_ID), Some(og_id)) => Ok(
                Self::ConfigDangerZonesButton(ConfigDangerZonesButton::try_from((data, og_id))?)
            ),
            _ => Err(TryFromInteractionError::UnknownCustomId {
                custom_id: String::from(&data.custom_id)
            })
        }
    }
}

#[derive(Debug)]
pub enum TryFromInteractionError {
    NoMessageComponent,
    UnknownCustomId {
        custom_id: String
    },
    IncorrectType {
        custom_id: String,
        expected: ComponentType,
        actual: ComponentType
    },
    ParseId {
        value: String
    }
}

impl Display for TryFromInteractionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TryFromInteractionError::NoMessageComponent => write!(f, "not a message component"),
            TryFromInteractionError::UnknownCustomId { custom_id } => {
                write!(f, "unknown custom id: {}", custom_id)
            }
            TryFromInteractionError::IncorrectType {
                custom_id,
                expected,
                actual
            } => write!(
                f,
                "incorrect type for custom id: {}, expected {}, but got {}",
                custom_id, expected, actual
            ),
            TryFromInteractionError::ParseId { value } => {
                write!(f, "could not parse interaction id {}", value)
            }
        }
    }
}

impl Error for TryFromInteractionError {}

pub async fn handle_message_component(
    interaction: Interaction,
    com: Arc<Com>
) -> anyhow::Result<()> {
    match MessageComponent::try_from(&interaction)? {
        MessageComponent::ConfigDangerZonesSelect(ConfigDangerZonesSelect {
            og_interaction_id,
            channels
        }) => config_danger_zones::action(interaction, og_interaction_id, channels, com).await?,
        MessageComponent::ConfigDangerZonesButton(ConfigDangerZonesButton {
            og_interaction_id
        }) => config_danger_zones::action(interaction, og_interaction_id, Vec::new(), com).await?
    }

    Ok(())
}
