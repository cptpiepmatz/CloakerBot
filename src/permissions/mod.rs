use std::error::Error;
use std::fmt::{Display, Formatter};

use twilight_model::guild::Permissions;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::permission_calculator::PermissionCalculator;

use crate::STATE;

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
    channel_id: Id<ChannelMarker>
) -> anyhow::Result<Permissions> {
    // get role ids
    let role_ids = {
        let event_cache = &STATE.event_cache;
        let member = event_cache.member(guild_id, user_id);
        match member {
            Some(member) => Vec::from(member.roles()),
            None => STATE
                .http_cache
                .fetch_member(guild_id, user_id)
                .await?
                .roles
                .to_vec()
        }
    };

    // get role permissions
    let mut role_permissions: Vec<(Id<RoleMarker>, Permissions)> = Vec::new();
    for role_id in role_ids.iter().cloned() {
        let role_perms = match STATE.event_cache.role(role_id) {
            Some(r) => r.permissions,
            None => {
                STATE
                    .http_cache
                    .fetch_role(guild_id, role_id)
                    .await?
                    .ok_or(PermSearchError::RoleNotInAllRoles)?
                    .permissions
            }
        };
        role_permissions.push((role_id, role_perms));
    }

    // get everyone role
    let everyone_perms = match STATE.event_cache.role(guild_id.cast()) {
        Some(r) => r.permissions,
        None => {
            STATE
                .http_cache
                .fetch_role(guild_id, guild_id.cast())
                .await?
                .ok_or(PermSearchError::EveryoneNotInCache)?
                .permissions
        }
    };

    // construct calculator
    let calc = PermissionCalculator::new(guild_id, user_id, everyone_perms, &role_permissions);

    // get channel overwrites
    let (channel_kind, channel_po) = match STATE.event_cache.channel(channel_id) {
        Some(c) => (c.kind, c.permission_overwrites.clone()),
        None => {
            let http_cache = &STATE.http_cache;
            let channel = http_cache
                .fetch_channel(guild_id, channel_id)
                .await?
                .ok_or(PermSearchError::ChannelNotInAllChannels)?;
            (channel.kind, channel.permission_overwrites.clone())
        }
    };
    Ok(calc.in_channel(channel_kind, &channel_po.unwrap_or_default()))
}
