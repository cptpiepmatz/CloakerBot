use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use futures::TryFutureExt;
use twilight_model::guild::{Permissions, Role};
use twilight_model::http::permission_overwrite::PermissionOverwrite;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
use twilight_model::id::Id;
use twilight_util::permission_calculator::PermissionCalculator;

use crate::Com;

#[derive(Debug)]
pub enum PermSearchError {
    EveryoneNotInCache,
    RoleNotInAllRoles
}

impl Display for PermSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PermSearchError::EveryoneNotInCache => write!(f, "everyone role is not in cache"),
            PermSearchError::RoleNotInAllRoles => write!(f, "role not found in all roles")
        }
    }
}

impl Error for PermSearchError {}

pub async fn get_in_channel_permissions(
    user_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
    channel_id: Id<ChannelMarker>,
    com: Arc<Com>
) -> anyhow::Result<Permissions> {
    // get role ids
    let event_cache_role_ids = || {
        com.event_cache
            .member(guild_id, user_id)
            .map(|m| Vec::from(m.value().roles()))
    };
    let http_cache_role_ids = || {
        com.http_cache
            .guild_members
            .get(&(guild_id, user_id))
            .map(|m| m.roles.clone())
    };
    let role_ids = match event_cache_role_ids().or_else(http_cache_role_ids) {
        Some(ids) => ids,
        None => {
            let member = com
                .http
                .guild_member(guild_id, user_id)
                .await?
                .model()
                .await?;
            let roles = member.roles.clone();
            com.http_cache
                .guild_members
                .insert((guild_id, user_id), member);
            roles
        }
    };

    // get role permissions
    let event_cache_role = |role_id| {
        com.event_cache
            .role(role_id)
            .map(|r| r.value().resource().clone())
    };
    let http_cache_role = |role_id| {
        com.http_cache
            .roles
            .get(&guild_id)
            .map(|g| g.value().get(&role_id).cloned())
            .flatten()
    };
    let mut role_permissions: Vec<(Id<RoleMarker>, Permissions)> = Vec::new();
    for role_id in role_ids {
        let role = match event_cache_role(role_id).or_else(|| http_cache_role(role_id)) {
            Some(role) => role,
            None => {
                let roles = com.http.roles(guild_id).await?.model().await?;
                let role = roles
                    .iter()
                    .find(|r| r.id == role_id)
                    .ok_or(PermSearchError::RoleNotInAllRoles)?
                    .clone();
                let pairs = roles.into_iter().map(|r| (r.id, r));
                let map: HashMap<Id<RoleMarker>, Role> = HashMap::from_iter(pairs);
                com.http_cache.roles.insert(guild_id, map);
                role
            }
        };
        role_permissions.push((role.id, role.permissions));
    }

    // get everyone role
    let everyone_role = event_cache_role(guild_id.cast())
        .or_else(|| http_cache_role(guild_id.cast()))
        .ok_or(PermSearchError::EveryoneNotInCache)?;

    // construct calculator
    let calc = PermissionCalculator::new(
        guild_id,
        user_id,
        everyone_role.permissions,
        &role_permissions
    );

    // get channel overwrites
    let event_cache_channel_perm_overwrites = || {
        com.event_cache
            .channel(channel_id)
            .map(|c| (c.permission_overwrites.clone(), c.kind))
    };
    let http_cache_channel_perm_overwrites = || {
        com.http_cache.channels.get(&guild_id).map(|g| {
            g.get(&channel_id)
                .map(|c| (c.permission_overwrites.clone(), c.kind))
        })
    };
    let (channel_po, channel_kind) = match (
        event_cache_channel_perm_overwrites(),
        http_cache_channel_perm_overwrites()
    ) {
        (Some((None, kind)), _) => (Vec::new(), kind),
        (Some((Some(po), kind)), _) => (po, kind),
        (_, Some(Some((None, kind)))) => (Vec::new(), kind),
        (_, Some(Some((Some(po), kind)))) => (po, kind),
        (_, _) => {
            let channels = com.http.guild_channels(guild_id).await?.model().await?;
            let channel = channels
                .iter()
                .find(|c| c.id == channel_id)
                .expect("find channel for requested channel id");
            let po = channel.permission_overwrites.clone().unwrap_or_default();
            let kind = channel.kind;
            let pairs = channels.into_iter().map(|c| (c.id, c));
            com.http_cache
                .channels
                .insert(guild_id, HashMap::from_iter(pairs));
            (po, kind)
        }
    };

    Ok(calc.in_channel(channel_kind, &channel_po))
}
