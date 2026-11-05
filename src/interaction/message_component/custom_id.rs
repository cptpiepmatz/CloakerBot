use std::str::FromStr;

use serde::{Deserialize, Serialize, Serializer};
use strum::{EnumString, IntoStaticStr};

use crate::interaction::message_component::HandleMessageComponentError;

#[derive(Debug, Copy, Clone, IntoStaticStr, EnumString, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(
    serialize_all = "snake_case",
    parse_err_ty = ParseMessageComponentCustomIdError,
    parse_err_fn = ParseMessageComponentCustomIdError::from_str,
)]
pub enum MessageComponentCustomId {
    OwnerNotificationsDisable,
    ConfigOpenDangerZones,
    ConfigOpenLogChannel,
    ConfigOpenImmunity,
    ConfigOpenRandom,
    ConfigOpenTargetBots,

    ConfigUnsetDangerZones,
    ConfigUnsetImmunityGeneral,
    ConfigUnsetImmunityRandom,
    ConfigUnsetImmunityGeneralSilent,
    ConfigUnsetImmunityRandomSilent,
    ConfigUnsetLogChannel,

    ConfigEnableRandom,
    ConfigDisableRandom,

    DangerZonesSelect,
    ImmunityGeneralSelect,
    ImmunityRandomSelect,
    ImmunityGeneralSilentSelect,
    ImmunityRandomSilentSelect,
    LogChannelSelect
}

impl ToString for MessageComponentCustomId {
    fn to_string(&self) -> String {
        let s: &'static str = self.into();
        s.into()
    }
}

pub struct ParseMessageComponentCustomIdError {
    unmatched: String
}

impl ParseMessageComponentCustomIdError {
    pub fn from_str(unmatched: &str) -> Self {
        Self { unmatched: unmatched.to_string() }
    }
}

impl From<ParseMessageComponentCustomIdError> for HandleMessageComponentError {
    fn from(value: ParseMessageComponentCustomIdError) -> Self {
        HandleMessageComponentError::UnknownCustomId(value.unmatched)
    }
}
