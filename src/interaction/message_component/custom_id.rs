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
    ImmunityRandomSelect,
    LogChannelSelect
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
const LOG_CHANNEL_SELECT: &str = "log_channel_select";

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
            ImmunityRandomSelect => IMMUNITY_RANDOM_SELECT,
            LogChannelSelect => LOG_CHANNEL_SELECT
        })
    }
}

pub struct ParseMessageComponentCustomIdError {
    unmatched: String
}

impl FromStr for MessageComponentCustomId {
    type Err = ParseMessageComponentCustomIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        type MCCID = MessageComponentCustomId;
        match s {
            CONFIG_UNSET_DANGER_ZONES => Ok(MCCID::ConfigUnsetDangerZones),
            CONFIG_UNSET_IMMUNITY_GENERAL => Ok(MCCID::ConfigUnsetImmunityGeneral),
            CONFIG_UNSET_IMMUNITY_RANDOM => Ok(MCCID::ConfigUnsetImmunityRandom),
            CONFIG_UNSET_LOG_CHANNEL => Ok(MCCID::ConfigUnsetLogChannel),
            CONFIG_UNSET_RANDOM => Ok(MCCID::ConfigUnsetRandom),
            CONFIG_UNSET_TARGET_BOTS => Ok(MCCID::ConfigUnsetTargetBots),

            DANGER_ZONES_SELECT => Ok(MCCID::DangerZonesSelect),
            IMMUNITY_GENERAL_SELECT => Ok(MCCID::ImmunityGeneralSelect),
            IMMUNITY_RANDOM_SELECT => Ok(MCCID::ImmunityRandomSelect),
            LOG_CHANNEL_SELECT => Ok(MCCID::LogChannelSelect),

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
