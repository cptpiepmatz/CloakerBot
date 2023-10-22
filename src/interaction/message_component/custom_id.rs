use std::str::FromStr;

use serde::{Serialize, Serializer};

use crate::interaction::message_component::HandleMessageComponentError;

#[derive(Debug, Copy, Clone)]
pub enum MessageComponentCustomId {
    ConfigUnsetDangerZones,
    ConfigUnsetImmunityGeneral,
    ConfigUnsetImmunityRandom,
    ConfigUnsetLogChannel,
    ConfigUnsetRandom,
    ConfigUnsetTargetBots,

    DangerZonesSelect,
    ImmunityGeneralSelect,
    ImmunityRandomSelect
}

const CONFIG_UNSET_DANGER_ZONES: &str = "config_unset_danger_zones";
const CONFIG_UNSET_IMMUNITY_GENERAL: &str = "config_unset_immunity_general";
const CONFIG_UNSET_IMMUNITY_RANDOM: &str = "config_unset_immunity_random";
const CONFIG_UNSET_LOG_CHANNEL: &str = "config_unset_log_channel";
const CONFIG_UNSET_RANDOM: &str = "config_unset_random";
const CONFIG_UNSET_TARGET_BOTS: &str = "config_unset_target_bots";

const DANGER_ZONES_SELECT: &str = "danger_zones_select";
const IMMUNITY_GENERAL_SELECT: &str = "immunity_general_select";
const IMMUNITY_RANDOM_SELECT: &str = "immunity_random_select";

impl ToString for MessageComponentCustomId {
    fn to_string(&self) -> String {
        use MessageComponentCustomId::*;
        String::from(match self {
            ConfigUnsetDangerZones => CONFIG_UNSET_DANGER_ZONES,
            ConfigUnsetImmunityGeneral => CONFIG_UNSET_IMMUNITY_GENERAL,
            ConfigUnsetImmunityRandom => CONFIG_UNSET_IMMUNITY_RANDOM,
            ConfigUnsetLogChannel => CONFIG_UNSET_LOG_CHANNEL,
            ConfigUnsetRandom => CONFIG_UNSET_RANDOM,
            ConfigUnsetTargetBots => CONFIG_UNSET_TARGET_BOTS,

            DangerZonesSelect => DANGER_ZONES_SELECT,
            ImmunityGeneralSelect => IMMUNITY_GENERAL_SELECT,
            ImmunityRandomSelect => IMMUNITY_RANDOM_SELECT
        })
    }
}

pub struct ParseMessageComponentCustomIdError {
    unmatched: String
}

impl FromStr for MessageComponentCustomId {
    type Err = ParseMessageComponentCustomIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use MessageComponentCustomId::*;
        match s {
            CONFIG_UNSET_DANGER_ZONES => Ok(ConfigUnsetDangerZones),
            CONFIG_UNSET_IMMUNITY_GENERAL => Ok(ConfigUnsetImmunityGeneral),
            CONFIG_UNSET_IMMUNITY_RANDOM => Ok(ConfigUnsetImmunityRandom),
            CONFIG_UNSET_LOG_CHANNEL => Ok(ConfigUnsetLogChannel),
            CONFIG_UNSET_RANDOM => Ok(ConfigUnsetRandom),
            CONFIG_UNSET_TARGET_BOTS => Ok(ConfigUnsetTargetBots),

            DANGER_ZONES_SELECT => Ok(DangerZonesSelect),
            IMMUNITY_GENERAL_SELECT => Ok(ImmunityGeneralSelect),
            IMMUNITY_RANDOM_SELECT => Ok(ImmunityRandomSelect),

            _ => Err(ParseMessageComponentCustomIdError {
                unmatched: s.to_owned()
            })
        }
    }
}

impl Serialize for MessageComponentCustomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        self.to_string().serialize(serializer)
    }
}

impl From<ParseMessageComponentCustomIdError> for HandleMessageComponentError {
    fn from(value: ParseMessageComponentCustomIdError) -> Self {
        HandleMessageComponentError::UnknownCustomId(value.unmatched)
    }
}
