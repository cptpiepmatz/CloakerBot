use std::str::FromStr;

use serde_json::json;
use twilight_mention::Mention;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::component::ComponentType;
use twilight_model::id::marker::{InteractionMarker, RoleMarker};
use twilight_model::id::Id;

use crate::db::config::GuildConfig;
use crate::db::DataStoreEntry;
use crate::interaction::message_component::TryFromInteractionError;
use crate::{APPLICATION_ID, STATE};

pub const SELECT_CUSTOM_ID: ConfigImmunityCustomId = ConfigImmunityCustomId {
    general: "cfg_im_gen_select",
    random: "cfg_im_rnd_select"
};
pub const BUTTON_CUSTOM_ID: ConfigImmunityCustomId = ConfigImmunityCustomId {
    general: "cfg_im_gen_button",
    random: "cfg_im_rnd_button"
};

pub struct ConfigImmunityCustomId {
    pub general: &'static str,
    pub random: &'static str
}

pub enum ConfigImmunityType {
    General,
    Random
}

pub struct ConfigImmunitySelect {
    pub og_interaction_id: Id<InteractionMarker>,
    pub kind: ConfigImmunityType,
    pub role: Option<Id<RoleMarker>>
}

pub struct ConfigImmunityButton {
    pub og_interaction_id: Id<InteractionMarker>,
    pub kind: ConfigImmunityType
}

impl TryFrom<(&MessageComponentInteractionData, &str, ConfigImmunityType)>
    for ConfigImmunitySelect
{
    type Error = TryFromInteractionError;

    fn try_from(
        (data, og_id, im_kind): (&MessageComponentInteractionData, &str, ConfigImmunityType)
    ) -> Result<Self, Self::Error> {
        match data.component_type {
            ComponentType::Unknown(6) => {}
            kind => {
                return Err(TryFromInteractionError::IncorrectType {
                    custom_id: match im_kind {
                        ConfigImmunityType::General => SELECT_CUSTOM_ID.general,
                        ConfigImmunityType::Random => SELECT_CUSTOM_ID.random
                    }
                    .into(),
                    expected: ComponentType::Unknown(6),
                    actual: kind
                })
            }
        }

        Ok(ConfigImmunitySelect {
            og_interaction_id: Id::from_str(og_id).map_err(|_| {
                TryFromInteractionError::ParseId {
                    value: og_id.to_owned()
                }
            })?,
            kind: im_kind,
            role: data
                .values
                .first()
                .map(|id| Id::from_str(id).expect("validated by Discord"))
        })
    }
}

impl TryFrom<(&MessageComponentInteractionData, &str, ConfigImmunityType)>
    for ConfigImmunityButton
{
    type Error = TryFromInteractionError;

    fn try_from(
        (data, og_id, im_kind): (&MessageComponentInteractionData, &str, ConfigImmunityType)
    ) -> Result<Self, Self::Error> {
        match data.component_type {
            ComponentType::Button => {}
            kind => {
                return Err(TryFromInteractionError::IncorrectType {
                    custom_id: match im_kind {
                        ConfigImmunityType::General => BUTTON_CUSTOM_ID.general,
                        ConfigImmunityType::Random => BUTTON_CUSTOM_ID.random
                    }
                    .into(),
                    expected: ComponentType::Button,
                    actual: kind
                })
            }
        }

        Ok(ConfigImmunityButton {
            og_interaction_id: Id::from_str(og_id).map_err(|_| {
                TryFromInteractionError::ParseId {
                    value: og_id.to_owned()
                }
            })?,
            kind: im_kind
        })
    }
}

pub async fn action(
    interaction: Interaction,
    og_interaction_id: Id<InteractionMarker>,
    role_id: Option<Id<RoleMarker>>,
    kind: ConfigImmunityType
) -> anyhow::Result<()> {
    use ConfigImmunityType::{General, Random};

    let og_interaction = match STATE.tracked_interactions.remove(&og_interaction_id) {
        Some((_, interaction)) => interaction,
        None => return Ok(())
    };

    let Some(guild_id) = interaction.guild_id else {
        // ignore if not in guild
        return Ok(())
    };

    let mut config = GuildConfig::select(guild_id).await?;

    let reply = match (&kind, role_id) {
        (General, None) => "**🛠️ Removed immune role for *all* takedowns.**".into(),
        (Random, None) => "**🛠️ Removed immune role for *random* takedowns.**".into(),
        (General, Some(role_id)) => format!(
            "**🛠️ Made {} immune to *all* takedowns.**",
            role_id.mention()
        ),
        (Random, Some(role_id)) => format!(
            "**🛠️ Made {} immune to *random* takedowns.**",
            role_id.mention()
        )
    };

    match kind {
        General => config.immunity.general = role_id,
        Random => config.immunity.random = role_id
    }
    config.update(guild_id).await?;

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
