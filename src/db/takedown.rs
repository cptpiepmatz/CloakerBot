use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::{NonZeroU64, ParseIntError};
use std::str::{FromStr, ParseBoolError};

use dashmap::DashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;

use crate::db::OptionalDataStoreEntry;

pub const TABLE_NAME: &str = "takedown";

lazy_static! {
    static ref CACHE: DashMap<TakedownPair, GuildTakedown> = DashMap::default();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TakedownPair {
    pub guild: Id<GuildMarker>,
    pub target: Id<UserMarker>,
    pub kind: TakedownType,
    pub countered: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildTakedown {
    pub guild: Id<GuildMarker>,
    pub target: Id<UserMarker>,
    pub kind: TakedownType,
    pub count: u32
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum TakedownType {
    User(Id<UserMarker>),
    Random,
    DangerZone
}

impl OptionalDataStoreEntry for GuildTakedown {
    type Key = TakedownPair;

    fn cache_get(key: &Self::Key) -> Option<Self> {
        CACHE.get(key).map(|r| r.clone())
    }

    fn cache_set(key: Self::Key, value: Option<Self>) {
        match value {
            Some(v) => {
                CACHE.insert(key, v);
            }
            None => {
                CACHE.remove(&key);
            }
        }
    }

    fn table_name() -> String {
        TABLE_NAME.into()
    }

    fn key(key: Self::Key) -> surrealdb::sql::Id {
        key.to_string().into()
    }
}

impl Display for TakedownType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TakedownType::User(i) => write!(f, "TTi{i}"),
            TakedownType::Random => write!(f, "TTr"),
            TakedownType::DangerZone => write!(f, "TTdz")
        }
    }
}

#[derive(Debug)]
pub enum TakedownTypeParseError {
    Parse(ParseIntError),
    InvalidInput(String)
}

impl Display for TakedownTypeParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TakedownTypeParseError::Parse(e) => write!(f, "{}", e),
            TakedownTypeParseError::InvalidInput(s) => write!(f, "invalid input {}", s)
        }
    }
}

impl Error for TakedownTypeParseError {}

impl FromStr for TakedownType {
    type Err = TakedownTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TTr" => Ok(TakedownType::Random),
            "TTdz" => Ok(TakedownType::DangerZone),
            s if s.starts_with("TTi") => {
                let id = s.split("TTi").nth(1);
                if let Some(id) = id {
                    let id: NonZeroU64 = id.parse().map_err(TakedownTypeParseError::Parse)?;
                    let id = Id::new(id.get());
                    return Ok(TakedownType::User(id));
                }
                Err(TakedownTypeParseError::InvalidInput(s.into()))
            }
            _ => Err(TakedownTypeParseError::InvalidInput(s.into()))
        }
    }
}

impl Display for TakedownPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "g{}_t{}_k{}_c{}",
            self.guild, self.target, self.kind, self.countered
        )
    }
}

pub enum TakedownPairParseError {
    Type(TakedownTypeParseError),
    IncorrectSchema(String),
    ParseInt(ParseIntError),
    ParseBool(ParseBoolError)
}

impl FromStr for TakedownPair {
    type Err = TakedownPairParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('_');
        let guild = split.next();
        let target = split.next();
        let kind = split.next();
        let countered = split.next();
        let (Some(guild), Some(target), Some(kind), Some(countered))
            = (guild, target, kind, countered) else {
            return Err(TakedownPairParseError::IncorrectSchema(s.into()));
        };
        let (Some(guild), Some(target), Some(kind), Some(countered)) = (
            guild.split('g').nth(1),
            target.split('t').nth(1),
            kind.split('k').nth(1),
            countered.split('c').nth(1)
        ) else {
            return Err(TakedownPairParseError::IncorrectSchema(s.into()));
        };

        let guild: NonZeroU64 = guild.parse().map_err(TakedownPairParseError::ParseInt)?;
        let target: NonZeroU64 = target.parse().map_err(TakedownPairParseError::ParseInt)?;
        let kind: TakedownType = kind.parse().map_err(TakedownPairParseError::Type)?;
        let countered: bool = countered
            .parse()
            .map_err(TakedownPairParseError::ParseBool)?;

        Ok(TakedownPair {
            guild: Id::new(guild.get()),
            target: Id::new(target.get()),
            kind,
            countered
        })
    }
}
