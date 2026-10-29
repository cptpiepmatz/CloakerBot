use std::fmt;

use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct SecretToken(String);

impl SecretToken {
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SecretToken {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err("invalid OAuth token");
        }
        Ok(Self(value))
    }
}

impl fmt::Debug for SecretToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretToken([redacted])")
    }
}

#[derive(Debug, Clone)]
pub struct StoredTokens {
    pub access_token: SecretToken,
    pub refresh_token: SecretToken,
    pub expires_at: DateTime<Utc>
}
