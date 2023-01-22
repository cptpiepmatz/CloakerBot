use std::error::Error;

use chrono::{DateTime, Utc};
use twilight_mention::timestamp::{Timestamp, TimestampStyle};
use twilight_mention::Mention;
use twilight_model::guild::Permissions;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::builder::embed::EmbedBuilder;

use crate::assets::colors;
use crate::db::config::GuildConfig;
use crate::db::DataStoreEntry;
use crate::permissions::fetch_in_channel_permissions;
use crate::{BOT_USER_ID, STATE};

pub enum LogMessage {
    UserTakedownStrike {
        target_id: Id<UserMarker>,
        initiator_id: Id<UserMarker>
    },
    RandomTakedownStrike {
        target_id: Id<UserMarker>,
        next: DateTime<Utc>
    },
    DangerZoneTakedownStrike {
        target_id: Id<UserMarker>,
        channel_id: Id<ChannelMarker>
    },
    UserTakedownCounter {
        target_id: Id<UserMarker>,
        initiator_id: Id<UserMarker>,
        countered_by_id: Id<UserMarker>
    },
    RandomTakedownCounter {
        target_id: Id<UserMarker>,
        countered_by_id: Id<UserMarker>,
        next: DateTime<Utc>
    },
    TakedownCheckNoStrike {
        next: DateTime<Utc>
    }
}

/// fetch a channel for logging, if no channel is found, return `None`
pub async fn fetch_log_channel(
    guild_id: Id<GuildMarker>
) -> anyhow::Result<Option<Id<ChannelMarker>>> {
    if let Some(log_channel_id) = GuildConfig::fetch_or_default(guild_id).await?.log_channel {
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

pub async fn log_message(
    _guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    message: LogMessage
) -> anyhow::Result<()> {
    let embed = EmbedBuilder::new().color(colors::PRIMARY);
    let embed = match message {
        LogMessage::UserTakedownStrike {
            target_id,
            initiator_id
        } => embed.title("👢 Takedown").description(format!(
            "{} was taken down by {}!",
            target_id.mention(),
            initiator_id.mention()
        )),
        LogMessage::RandomTakedownStrike { target_id, next } => {
            embed.title("👢 Random Takedown").description(format!(
                "{} was randomly taken down!\nNext check will be {}.",
                target_id.mention(),
                Timestamp::new(next.timestamp() as u64, Some(TimestampStyle::RelativeTime))
                    .mention()
            ))
        }
        LogMessage::DangerZoneTakedownStrike {
            target_id,
            channel_id
        } => embed.title("👢 Danger Zone Takedown").description(format!(
            "{} was taken down in the danger zone {}!",
            target_id.mention(),
            channel_id.mention()
        )),
        LogMessage::UserTakedownCounter {
            target_id,
            initiator_id,
            countered_by_id
        } => embed.title("☂️ Takedown Counter").description(format!(
            "{} countered takedown targeted at {}, initiated by {}!",
            countered_by_id.mention(),
            target_id.mention(),
            initiator_id.mention()
        )),
        LogMessage::RandomTakedownCounter {
            target_id,
            countered_by_id,
            next
        } => embed
            .title("☂️ Random Takedown Counter")
            .description(format!(
                "{} countered random takedown targeted at {}!\nNext check will be {}.",
                countered_by_id.mention(),
                target_id.mention(),
                Timestamp::new(next.timestamp() as u64, Some(TimestampStyle::RelativeTime))
                    .mention()
            )),
        LogMessage::TakedownCheckNoStrike { next } => {
            embed.title("🎲 No Random Takedown").description(format!(
                "Not enough people in voice chats.\nNext check will be {}.",
                Timestamp::new(next.timestamp() as u64, Some(TimestampStyle::RelativeTime))
                    .mention()
            ))
        }
    };

    STATE
        .http
        .create_message(channel_id)
        .embeds(&[embed.build()])?
        .await?;

    Ok(())
}
