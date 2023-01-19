use std::ops::Deref;

use dashmap::mapref::one::Ref as HttpRef;
use twilight_cache_inmemory::model::CachedMember;
use twilight_cache_inmemory::Reference as EventRef;
use twilight_model::guild::Member;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::user::User;
use twilight_model::util::{ImageHash, Timestamp};

type HttpReference<'r> = HttpRef<'r, (Id<GuildMarker>, Id<UserMarker>), Member>;
type EventReference<'r> = (
    EventRef<'r, (Id<GuildMarker>, Id<UserMarker>), CachedMember>,
    EventRef<'r, Id<UserMarker>, User>
);

pub enum RefCachedMember<'r> {
    Event(EventReference<'r>),
    Http(HttpReference<'r>)
}

impl<'r> RefCachedMember<'r> {
    pub fn avatar(&self) -> Option<ImageHash> {
        match self {
            Self::Event(r) => r.0.avatar(),
            Self::Http(r) => r.avatar
        }
    }

    pub fn communication_disabled_until(&self) -> Option<Timestamp> {
        match self {
            Self::Event(r) => r.0.communication_disabled_until(),
            Self::Http(r) => r.communication_disabled_until
        }
    }

    // pub fn deaf(&self) -> Option<bool> {
    //     match self {
    //         Self::Event(r) => r.0.deaf(),
    //         Self::Http(r) => r.deaf
    //     }
    // }

    pub fn guild_id(&self) -> Id<GuildMarker> {
        match self {
            Self::Event(r) => r.0.guild_id(),
            Self::Http(r) => r.guild_id
        }
    }

    pub fn joined_at(&self) -> Timestamp {
        match self {
            Self::Event(r) => r.0.joined_at(),
            Self::Http(r) => r.joined_at
        }
    }

    // pub fn mute(&self) -> Option<bool> {
    //     match self {
    //         Self::Event(r) => r.0.mute(),
    //         Self::Http(r) => r.mute
    //     }
    // }

    pub fn nick(&self) -> Option<&str> {
        match self {
            Self::Event(r) => r.0.nick(),
            Self::Http(r) => r.nick.as_deref()
        }
    }

    pub fn pending(&self) -> bool {
        match self {
            Self::Event(r) => r.0.pending(),
            Self::Http(r) => r.pending
        }
    }

    pub fn premium_since(&self) -> Option<Timestamp> {
        match self {
            Self::Event(r) => r.0.premium_since(),
            Self::Http(r) => r.premium_since
        }
    }

    //roles

    //user_id

    pub fn user(&self) -> &User {
        match self {
            Self::Event(r) => r.1.deref(),
            Self::Http(r) => &r.user
        }
    }
}

impl<'r> From<EventReference<'r>> for RefCachedMember<'r> {
    fn from(value: EventReference<'r>) -> Self {
        RefCachedMember::Event(value)
    }
}

impl<'r> From<HttpReference<'r>> for RefCachedMember<'r> {
    fn from(value: HttpReference<'r>) -> Self {
        RefCachedMember::Http(value)
    }
}
