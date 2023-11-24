use derive_more::From;
use itertools::Itertools;

use crate::data::config::GuildConfig;
use crate::takedown::{takedown, TakedownFail, TakedownResult, TakedownType};
use crate::util::cache::VoiceChannelCacheUpdate;
use crate::util::ClientError;
use crate::STATE;

#[derive(Debug, From)]
pub enum CheckDangerZonesError {
    Client(ClientError)
}

pub async fn check_danger_zones(
    update: VoiceChannelCacheUpdate
) -> Result<(), CheckDangerZonesError> {
    let Some(channel_id) = update.new
    else {
        return Ok(());
    };

    let guild_configs = STATE.guild_configs.read().await;
    let config = GuildConfig::default();
    let config = guild_configs.get(&update.guild_id).unwrap_or(&config);

    let danger_zones = &config.danger_zones;
    if !danger_zones.contains(&channel_id) {
        return Ok(());
    }

    match takedown(update.user_id, update.guild_id, TakedownType::DangerZone {
        channel_id
    })
    .await
    {
        Ok(_) => Ok(()),
        Err(TakedownFail::Client(err)) => Err(err.into()),
        Err(_) => Ok(())
    }
}
