use strum::{EnumString, IntoStaticStr};

use super::HandleModalSubmitError;

#[derive(Debug, Copy, Clone, IntoStaticStr, EnumString)]
#[allow(clippy::enum_variant_names, reason = "Custom IDs include their configuration namespace")]
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

impl std::fmt::Display for ModalCustomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: &'static str = self.into();
        f.write_str(value)
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
