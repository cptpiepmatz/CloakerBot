use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::event::Event;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::id::marker::GenericMarker;
use twilight_model::id::Id;

use crate::data::persist::Persist;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistentNameCache(HashMap<Id<GenericMarker>, String>);

impl PersistentNameCache {
    pub fn update(&mut self, event: &Event) -> bool {
        match event {
            Event::ChannelCreate(v) => self.insert(v.id, v.name.as_ref()),
            Event::ChannelUpdate(v) => self.insert(v.id, v.name.as_ref()),
            Event::GuildCreate(v) => self.insert(v.id, &v.name),
            Event::GuildUpdate(v) => self.insert(v.id, &v.name),
            Event::MemberAdd(v) => self.insert(v.member.user.id, &v.member.user.name),
            Event::MemberUpdate(v) => self.insert(v.user.id, &v.user.name),
            Event::RoleCreate(v) => self.insert(v.role.id, &v.role.name),
            Event::RoleUpdate(v) => self.insert(v.role.id, &v.role.name),

            Event::InteractionCreate(v) => self.handle_interaction_create(v.as_ref()),

            _ => false
        }
    }

    fn insert<'s, M>(&mut self, id: Id<M>, name: impl Into<Option<&'s String>>) -> bool {
        let id = id.cast();

        let Some(name) = name.into() else { return false; };
        let name = name.trim();
        if name.is_empty() {
            return false;
        }

        match self.0.get(&id) {
            None => self.0.insert(id, name.to_owned()),
            Some(current) if current != name => self.0.insert(id, name.to_owned()),
            _ => return false
        };

        true
    }

    fn handle_interaction_create(&mut self, interaction_create: &InteractionCreate) -> bool {
        let mut did_update = false;

        if let Some(ref user) = interaction_create.user {
            did_update |= self.insert(user.id, &user.name);
        }

        if let Some(ref channel) = interaction_create.channel {
            did_update |= self.insert(channel.id, channel.name.as_ref());
        }

        if let Some(InteractionData::ApplicationCommand(ref data)) = interaction_create.data {
            if let Some(ref resolved) = data.resolved {
                did_update |= self.map_insert(&resolved.channels, |ch| &ch.name);
                did_update |= self.map_insert(&resolved.users, |user| &user.name);
                did_update |= self.map_insert(&resolved.roles, |role| &role.name);
            }
        }

        did_update
    }

    fn map_insert<'m, M, T: 'm, S>(
        &mut self,
        map: &'m HashMap<Id<M>, T>,
        extract: impl Fn(&'m T) -> S
    ) -> bool
    where
        S: Into<&'m String>
    {
        map.iter()
            .map(|(&id, v)| self.insert(id, extract(v).into()))
            .reduce(|acc, v| acc || v)
            .unwrap_or(false)
    }
}

impl Persist for PersistentNameCache {
    type Item = Self;

    const DATA_PATH: &'static str = "./data/names.json";
}
