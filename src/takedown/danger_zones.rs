use derive_more::From;
use infra::util::ClientError;
use infra::util::cache::VoiceChannelCacheUpdate;
use itertools::Itertools;
use macros::require;

use crate::data::config::GuildConfig;
use crate::takedown::{TakedownFail, TakedownResult, TakedownType, takedown};
use crate::{BOT_USER_ID, STATE};

#[derive(Debug, From)]
pub enum CheckDangerZonesError {
    Database(sqlx::Error),
    Client(ClientError)
}

pub async fn check_danger_zones(
    update: VoiceChannelCacheUpdate
) -> Result<(), CheckDangerZonesError> {
    let channel_id = require!(Some(update.new) else Ok(()));

    let is_danger_zone =
        STATE.database.guild_config(update.guild_id).await?.danger_zones.contains(&channel_id);
    require!(is_danger_zone else Ok(()));

    require!(update.user_id != BOT_USER_ID else Ok(()));

    match takedown(update.user_id, update.guild_id, TakedownType::DangerZone { channel_id }).await {
        Err(TakedownFail::Client(err)) => Err(err.into()),
        Err(TakedownFail::Database(err)) => Err(err.into()),
        Err(_) | Ok(_) => Ok(())
    }
}
