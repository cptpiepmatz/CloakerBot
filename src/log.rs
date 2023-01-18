use std::sync::Arc;

use twilight_model::guild::Permissions;
use twilight_model::id::marker::{ChannelMarker, GuildMarker};
use twilight_model::id::Id;

use crate::db::config::GuildConfig;
use crate::permissions::fetch_in_channel_permissions;
use crate::{db, State, BOT_USER_ID};

/// fetch a channel for logging, if no channel is found, return `None`
pub async fn fetch_log_channel(
    guild_id: Id<GuildMarker>,
    com: Arc<State>
) -> anyhow::Result<Option<Id<ChannelMarker>>> {
    let guild_config: Option<GuildConfig> = com
        .db
        .select((db::config::TABLE_NAME, guild_id.get()))
        .await?;
    if let Some(GuildConfig {
        log_channel: Some(log_channel_id),
        ..
    }) = guild_config
    {
        let channel_perms =
            fetch_in_channel_permissions(BOT_USER_ID, guild_id, log_channel_id, com.clone())
                .await?;
        if channel_perms.contains(Permissions::SEND_MESSAGES) {
            return Ok(Some(log_channel_id));
        }
    }

    match com.event_cache.guild(guild_id) {
        Some(cached_guild) => {
            // TODO: check `public_updates_channel_id` when in twilight_model::guild::Guild
            if let Some(system_channel_id) = cached_guild.system_channel_id() {
                let channel_perms = fetch_in_channel_permissions(
                    BOT_USER_ID,
                    guild_id,
                    system_channel_id,
                    com.clone()
                )
                .await?;
                if channel_perms.contains(Permissions::SEND_MESSAGES) {
                    return Ok(Some(system_channel_id));
                }
            }
        }
        None => {
            let guild = com.http_cache.fetch_guild(guild_id, com.clone()).await?;
            if let Some(system_channel_id) = guild.system_channel_id {
                let channel_perms = fetch_in_channel_permissions(
                    BOT_USER_ID,
                    guild_id,
                    system_channel_id,
                    com.clone()
                )
                .await?;
                if channel_perms.contains(Permissions::SEND_MESSAGES) {
                    return Ok(Some(system_channel_id));
                }
            }
        }
    }

    match com.event_cache.guild_channels(guild_id) {
        Some(channels) => {
            for channel_id in channels.value().iter().cloned() {
                if let Some(channel) = com.event_cache.channel(channel_id) {
                    if let Some(name) = &channel.name {
                        if name == "general" {
                            let channel_perms = fetch_in_channel_permissions(
                                BOT_USER_ID,
                                guild_id,
                                channel_id,
                                com.clone()
                            )
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
            for channel in com
                .http_cache
                .fetch_guild_channels(guild_id, com.clone())
                .await?
                .value()
            {
                if let Some(name) = &channel.name {
                    if name == "general" {
                        let channel_perms = fetch_in_channel_permissions(
                            BOT_USER_ID,
                            guild_id,
                            channel.id,
                            com.clone()
                        )
                        .await?;
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
