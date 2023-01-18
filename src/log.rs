use twilight_model::guild::Permissions;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;

use crate::db::config::GuildConfig;
use crate::db::DataStoreEntry;
use crate::permissions::fetch_in_channel_permissions;
use crate::{BOT_USER_ID, STATE};

/// fetch a channel for logging, if no channel is found, return `None`
pub async fn fetch_log_channel(
    guild_id: Id<GuildMarker>
) -> anyhow::Result<Option<Id<ChannelMarker>>> {
    if let Some(log_channel_id) = GuildConfig::fetch(guild_id).await?.log_channel {
        let channel_perms =
            fetch_in_channel_permissions(BOT_USER_ID, guild_id, log_channel_id).await?;
        if channel_perms.contains(Permissions::SEND_MESSAGES) {
            return Ok(Some(log_channel_id));
        }
    }

    match STATE.event_cache.guild(guild_id) {
        Some(cached_guild) => {
            // TODO: check `public_updates_channel_id` when in twilight_model::guild::Guild
            if let Some(system_channel_id) = cached_guild.system_channel_id() {
                let channel_perms =
                    fetch_in_channel_permissions(BOT_USER_ID, guild_id, system_channel_id).await?;
                if channel_perms.contains(Permissions::SEND_MESSAGES) {
                    return Ok(Some(system_channel_id));
                }
            }
        }
        None => {
            let http_cache = &STATE.http_cache;
            let guild = http_cache.fetch_guild(guild_id).await?;
            if let Some(system_channel_id) = guild.system_channel_id {
                let channel_perms =
                    fetch_in_channel_permissions(BOT_USER_ID, guild_id, system_channel_id).await?;
                if channel_perms.contains(Permissions::SEND_MESSAGES) {
                    return Ok(Some(system_channel_id));
                }
            }
        }
    }

    match STATE.event_cache.guild_channels(guild_id) {
        Some(channels) => {
            for channel_id in channels.value().iter().cloned() {
                if let Some(channel) = STATE.event_cache.channel(channel_id) {
                    if let Some(name) = &channel.name {
                        if name == "general" {
                            let channel_perms =
                                fetch_in_channel_permissions(BOT_USER_ID, guild_id, channel_id)
                                    .await?;
                            if channel_perms.contains(Permissions::SEND_MESSAGES) {
                                return Ok(Some(channel_id));
                            }
                        }
                    }
                }
            }
        }
        None => {
            for channel in STATE
                .http_cache
                .fetch_guild_channels(guild_id)
                .await?
                .value()
            {
                if let Some(name) = &channel.name {
                    if name == "general" {
                        let channel_perms =
                            fetch_in_channel_permissions(BOT_USER_ID, guild_id, channel.id).await?;
                        if channel_perms.contains(Permissions::SEND_MESSAGES) {
                            return Ok(Some(channel.id));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
