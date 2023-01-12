
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;


use twilight_model::guild::{Permissions};
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::permission_calculator::PermissionCalculator;

use crate::Com;

#[derive(Debug)]
pub enum PermSearchError {
    EveryoneNotInCache,
    RoleNotInAllRoles,
    ChannelNotInAllChannels
}

impl Display for PermSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PermSearchError::EveryoneNotInCache => write!(f, "everyone role is not in cache"),
            PermSearchError::RoleNotInAllRoles => write!(f, "role not found in all roles"),
            PermSearchError::ChannelNotInAllChannels => {
                write!(f, "channel not found in all channels")
            }
        }
    }
}

impl Error for PermSearchError {}

pub async fn fetch_in_channel_permissions(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    com: Arc<Com>
) -> anyhow::Result<Permissions> {
    // get role ids
    let role_ids = {
        let member = com.event_cache.member(guild_id, user_id);
        match member {
            Some(member) => Vec::from(member.roles()),
            None => com
                .http_cache
                .fetch_member(guild_id, user_id, com.clone())
                .await?
                .roles
                .iter()
                .cloned()
                .collect()
        }
    };

    // get role permissions
    let mut role_permissions: Vec<(Id<RoleMarker>, Permissions)> = Vec::new();
    for role_id in role_ids.iter().cloned() {
        let role_perms = match com.event_cache.role(role_id) {
            Some(r) => r.permissions,
            None => {
                com.http_cache
                    .fetch_role(guild_id, role_id, com.clone())
                    .await?
                    .ok_or(PermSearchError::RoleNotInAllRoles)?
                    .permissions
            }
        };
        role_permissions.push((role_id, role_perms));
    }

    // get everyone role
    let everyone_perms = match com.event_cache.role(guild_id.cast()) {
        Some(r) => r.permissions,
        None => {
            com.http_cache
                .fetch_role(guild_id, guild_id.cast(), com.clone())
                .await?
                .ok_or(PermSearchError::EveryoneNotInCache)?
                .permissions
        }
    };

    // construct calculator
    let calc = PermissionCalculator::new(guild_id, user_id, everyone_perms, &role_permissions);

    // get channel overwrites
    let (channel_kind, channel_po) = match com.event_cache.channel(channel_id) {
        Some(c) => (c.kind, c.permission_overwrites.clone()),
        None => {
            let channel = com
                .http_cache
                .fetch_channel(guild_id, channel_id, com.clone())
                .await?
                .ok_or(PermSearchError::ChannelNotInAllChannels)?;
            (channel.kind, channel.permission_overwrites.clone())
        }
    };
    Ok(calc.in_channel(channel_kind, &channel_po.unwrap_or_default()))
}
