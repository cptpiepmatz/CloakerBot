use std::str::FromStr;

use serde::{Serialize, Serializer};

use crate::interaction::message_component::HandleMessageComponentError;

#[derive(Debug, Copy, Clone)]
pub enum MessageComponentCustomId {
    ConfigUnsetDangerZones,
    ConfigUnsetImmunityGeneral,
    ConfigUnsetImmunityRandom,
    ConfigUnsetImmunityGeneralSilent,
    ConfigUnsetImmunityRandomSilent,
    ConfigUnsetLogChannel,
    ConfigUnsetRandom,

    DangerZonesSelect,
    ImmunityGeneralSelect,
    ImmunityRandomSelect,
    ImmunityGeneralSilentSelect,
    ImmunityRandomSilentSelect,
    LogChannelSelect
}

const CONFIG_UNSET_DANGER_ZONES: &str = "config_unset_danger_zones";
const CONFIG_UNSET_IMMUNITY_GENERAL: &str = "config_unset_immunity_general";
const CONFIG_UNSET_IMMUNITY_RANDOM: &str = "config_unset_immunity_random";
const CONFIG_UNSET_IMMUNITY_GENERAL_SILENT: &str = "config_unset_immunity_general_silent";
const CONFIG_UNSET_IMMUNITY_RANDOM_SILENT: &str = "config_unset_immunity_random_silent";
const CONFIG_UNSET_LOG_CHANNEL: &str = "config_unset_log_channel";
const CONFIG_UNSET_RANDOM: &str = "config_unset_random";

const DANGER_ZONES_SELECT: &str = "danger_zones_select";
const IMMUNITY_GENERAL_SELECT: &str = "immunity_general_select";
const IMMUNITY_RANDOM_SELECT: &str = "immunity_random_select";
const IMMUNITY_GENERAL_SILENT_SELECT: &str = "immunity_general_silent_select";
const IMMUNITY_RANDOM_SILENT_SELECT: &str = "immunity_random_silent_select";
const LOG_CHANNEL_SELECT: &str = "log_channel_select";

impl ToString for MessageComponentCustomId {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::ConfigUnsetDangerZones => CONFIG_UNSET_DANGER_ZONES,
            Self::ConfigUnsetImmunityGeneral => CONFIG_UNSET_IMMUNITY_GENERAL,
            Self::ConfigUnsetImmunityRandom => CONFIG_UNSET_IMMUNITY_RANDOM,
            Self::ConfigUnsetImmunityGeneralSilent => CONFIG_UNSET_IMMUNITY_GENERAL_SILENT,
            Self::ConfigUnsetImmunityRandomSilent => CONFIG_UNSET_IMMUNITY_RANDOM_SILENT,
            Self::ConfigUnsetLogChannel => CONFIG_UNSET_LOG_CHANNEL,
            Self::ConfigUnsetRandom => CONFIG_UNSET_RANDOM,

            Self::DangerZonesSelect => DANGER_ZONES_SELECT,
            Self::ImmunityGeneralSelect => IMMUNITY_GENERAL_SELECT,
            Self::ImmunityRandomSelect => IMMUNITY_RANDOM_SELECT,
            Self::ImmunityGeneralSilentSelect => IMMUNITY_GENERAL_SILENT_SELECT,
            Self::ImmunityRandomSilentSelect => IMMUNITY_RANDOM_SILENT_SELECT,
            Self::LogChannelSelect => LOG_CHANNEL_SELECT
        })
    }
}

pub struct ParseMessageComponentCustomIdError {
    unmatched: String
}

impl FromStr for MessageComponentCustomId {
    type Err = ParseMessageComponentCustomIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        type MCCId = MessageComponentCustomId;
        match s {
            CONFIG_UNSET_DANGER_ZONES => Ok(MCCId::ConfigUnsetDangerZones),
            CONFIG_UNSET_IMMUNITY_GENERAL => Ok(MCCId::ConfigUnsetImmunityGeneral),
            CONFIG_UNSET_IMMUNITY_RANDOM => Ok(MCCId::ConfigUnsetImmunityRandom),
            CONFIG_UNSET_IMMUNITY_GENERAL_SILENT => Ok(MCCId::ConfigUnsetImmunityGeneralSilent),
            CONFIG_UNSET_IMMUNITY_RANDOM_SILENT => Ok(MCCId::ConfigUnsetImmunityRandomSilent),
            CONFIG_UNSET_LOG_CHANNEL => Ok(MCCId::ConfigUnsetLogChannel),
            CONFIG_UNSET_RANDOM => Ok(MCCId::ConfigUnsetRandom),

            DANGER_ZONES_SELECT => Ok(MCCId::DangerZonesSelect),
            IMMUNITY_GENERAL_SELECT => Ok(MCCId::ImmunityGeneralSelect),
            IMMUNITY_RANDOM_SELECT => Ok(MCCId::ImmunityRandomSelect),
            IMMUNITY_GENERAL_SILENT_SELECT => Ok(MCCId::ImmunityGeneralSilentSelect),
            IMMUNITY_RANDOM_SILENT_SELECT => Ok(MCCId::ImmunityRandomSilentSelect),
            LOG_CHANNEL_SELECT => Ok(MCCId::LogChannelSelect),

            _ => Err(ParseMessageComponentCustomIdError { unmatched: s.to_owned() })
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
