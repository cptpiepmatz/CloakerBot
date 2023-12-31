use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::data::{Persist, PersistentData};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GuildConfig {
    pub danger_zones: Vec<Id<ChannelMarker>>,
    pub immunity: GuildConfigImmunity,
    pub log_channel: Option<Id<ChannelMarker>>,
    pub random: GuildConfigRandom
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GuildConfigImmunity {
    pub general: Vec<Id<RoleMarker>>,
    pub random: Vec<Id<RoleMarker>>,
    pub general_silent: Vec<Id<RoleMarker>>,
    pub random_silent: Vec<Id<RoleMarker>>
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GuildConfigRandom {
    pub enabled: Option<bool>,
    pub min: Option<f64>,
    pub max: Option<f64>
}

#[derive(Debug, SmartDefault)]
pub struct CompleteGuildConfigRandom {
    #[default = false]
    pub enabled: bool,
    #[default = 1.0]
    pub min: f64,
    #[default = 24.0]
    pub max: f64
}

impl CompleteDefault for GuildConfigRandom {
    type Complete = CompleteGuildConfigRandom;

    fn complete(&self) -> Self::Complete {
        let default = Self::Complete::default();
        Self::Complete {
            enabled: self.enabled.unwrap_or(default.enabled),
            min: self.min.unwrap_or(default.min),
            max: self.max.unwrap_or(default.max)
        }
    }
}

pub type PersistentGuildConfig = HashMap<Id<GuildMarker>, GuildConfig>;
impl Persist for PersistentGuildConfig {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/guild_configs.json";
}

/// A trait for converting an instance of a type with optional fields into a
/// complete version with all fields filled.
pub trait CompleteDefault {
    /// The complete version of the type, with no optional fields.
    ///
    /// This associated type represents a version of the original type where all
    /// [`Option<T>`] fields are replaced with their non-optional
    /// counterparts (`T`). It must implement [`Default`] to provide default
    /// values for its fields.
    type Complete: Default;

    /// Converts the current instance into its complete version, filling in
    /// `None` values with defaults.
    ///
    /// This method returns an instance of the `Complete` type, where any
    /// `Option::None` fields in the original instance are replaced with
    /// their default values.
    fn complete(&self) -> Self::Complete;
}
