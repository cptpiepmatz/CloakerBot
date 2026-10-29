use derive_more::From;
use itertools::Itertools;

use crate::STATE;
use crate::data::config::GuildConfig;
use crate::takedown::{TakedownFail, TakedownResult, TakedownType, takedown};
use crate::util::cache::VoiceChannelCacheUpdate;
use crate::util::{ClientError, require};

#[derive(Debug, From)]
pub enum CheckDangerZonesError {
    Client(ClientError)
}

pub async fn check_danger_zones(
    update: VoiceChannelCacheUpdate
) -> Result<(), CheckDangerZonesError> {
    let channel_id = require!(Some(update.new) => Ok(()));

    let is_danger_zone = STATE
        .guild_configs
        .read()
        .await
        .get(&update.guild_id)
        .is_some_and(|config| config.danger_zones.contains(&channel_id));
    require!(is_danger_zone => Ok(()));

    match takedown(update.user_id, update.guild_id, TakedownType::DangerZone { channel_id }).await {
        Err(TakedownFail::Client(err)) => Err(err.into()),
        Err(_) | Ok(_) => Ok(())
    }
}
