use derive_more::From;
use itertools::Itertools;

use crate::data::config::GuildConfig;
use crate::takedown::{takedown, TakedownFail, TakedownResult, TakedownType};
use crate::util::cache::VoiceChannelCacheUpdate;
use crate::util::{ClientError, ifr, ifr_some};
use crate::STATE;

#[derive(Debug, From)]
pub enum CheckDangerZonesError {
    Client(ClientError)
}

pub async fn check_danger_zones(
    update: VoiceChannelCacheUpdate
) -> Result<(), CheckDangerZonesError> {
    let channel_id = ifr_some!(update.new => Ok(()));

    let guild_configs = STATE.guild_configs.read().await;
    let config = GuildConfig::default();
    let config = guild_configs.get(&update.guild_id).unwrap_or(&config);

    let danger_zones = &config.danger_zones;
    ifr!(danger_zones.contains(&channel_id) => Ok(()));

    match takedown(update.user_id, update.guild_id, TakedownType::DangerZone {
        channel_id
    })
    .await
    {
        Err(TakedownFail::Client(err)) => Err(err.into()),
        Err(_) | Ok(_) => Ok(())
    }
}
