pub mod http;
pub mod twilight;

pub mod event {
    use twilight_cache_inmemory::{InMemoryCache as EventCache, ResourceType};

    pub fn build_cache() -> EventCache {
        EventCache::builder()
            .resource_types(EVENT_CACHE_RESOURCE_TYPES)
            .build()
    }

    pub const EVENT_CACHE_RESOURCE_TYPES: ResourceType = ResourceType::VOICE_STATE
        .union(ResourceType::CHANNEL)
        .union(ResourceType::MEMBER)
        .union(ResourceType::ROLE)
        .union(ResourceType::USER)
        .union(ResourceType::GUILD);
}
