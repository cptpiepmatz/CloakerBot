use strum::{EnumString, IntoStaticStr};

use super::HandleModalSubmitError;

#[derive(Debug, Copy, Clone, IntoStaticStr, EnumString)]
#[strum(
    serialize_all = "snake_case",
    parse_err_ty = ParseModalCustomIdError,
    parse_err_fn = ParseModalCustomIdError::from_str,
)]
pub enum ModalCustomId {
    ConfigDangerZones,
    ConfigLogChannel,
    ConfigImmunity,
    ConfigRandom,
    ConfigTargetBots
}

impl ToString for ModalCustomId {
    fn to_string(&self) -> String {
        let value: &'static str = self.into();
        value.into()
    }
}

pub struct ParseModalCustomIdError {
    unmatched: String
}

impl ParseModalCustomIdError {
    pub fn from_str(unmatched: &str) -> Self {
        Self { unmatched: unmatched.to_string() }
    }
}

impl From<ParseModalCustomIdError> for HandleModalSubmitError {
    fn from(value: ParseModalCustomIdError) -> Self {
        Self::UnknownCustomId(value.unmatched)
    }
}
