pub mod http {
    
    use std::sync::Arc;

    use dashmap::mapref::one::Ref;
    use dashmap::DashMap;
    use twilight_model::channel::Channel;
    use twilight_model::guild::{Guild, Member, Role};
    use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};
    use twilight_model::id::Id;
    use twilight_model::user::User;

    use crate::Com;

    #[derive(Debug, Default)]
    pub struct Cache {
        users: DashMap<Id<UserMarker>, User>,
        members: DashMap<(Id<GuildMarker>, Id<UserMarker>), Member>,
        roles: DashMap<Id<RoleMarker>, Role>,
        channels: DashMap<Id<ChannelMarker>, Channel>,
        guild_channels: DashMap<Id<GuildMarker>, Vec<Channel>>,
        guilds: DashMap<Id<GuildMarker>, Guild>
    }

    impl Cache {
        pub async fn fetch_user(
            &self,
            user_id: Id<UserMarker>,
            com: Arc<Com>
        ) -> anyhow::Result<Ref<Id<UserMarker>, User>> {
            if let Some(user) = self.users.get(&user_id) {
                return Ok(user);
            }

            let user = com.http.user(user_id).await?.model().await?;
            self.users.insert(user_id, user);

            Ok(self.users.get(&user_id).expect("just inserted"))
        }

        pub async fn fetch_member(
            &self,
            guild_id: Id<GuildMarker>,
            user_id: Id<UserMarker>,
            com: Arc<Com>
        ) -> anyhow::Result<Ref<(Id<GuildMarker>, Id<UserMarker>), Member>> {
            if let Some(member) = self.members.get(&(guild_id, user_id)) {
                return Ok(member);
            }

            let member = com
                .http
                .guild_member(guild_id, user_id)
                .await?
                .model()
                .await?;
            self.users.insert(user_id, member.user.clone());
            self.members.insert((guild_id, user_id), member);

            Ok(self
                .members
                .get(&(guild_id, user_id))
                .expect("just inserted"))
        }

        pub async fn fetch_role(
            &self,
            guild_id: Id<GuildMarker>,
            role_id: Id<RoleMarker>,
            com: Arc<Com>
        ) -> anyhow::Result<Option<Ref<Id<RoleMarker>, Role>>> {
            if let Some(role) = self.roles.get(&role_id) {
                return Ok(Some(role));
            }

            let roles = com.http.roles(guild_id).await?.model().await?;
            for role in roles {
                self.roles.insert(role.id, role);
            }

            Ok(self.roles.get(&role_id))
        }
        
        pub async fn fetch_channel(
            &self,
            guild_id: Id<GuildMarker>,
            channel_id: Id<ChannelMarker>,
            com: Arc<Com>
        ) -> anyhow::Result<Option<Ref<Id<ChannelMarker>, Channel>>> {
            if let Some(channel) = self.channels.get(&channel_id) {
                return Ok(Some(channel));
            }
            
            let channels = com.http.guild_channels(guild_id).await?.model().await?;
            for channel in channels.iter().cloned() {
                self.channels.insert(channel.id, channel);
            }
            self.guild_channels.insert(guild_id, channels);
            
            Ok(self.channels.get(&channel_id))
        }
        
        pub async fn fetch_guild(&self, guild_id: Id<GuildMarker>, com: Arc<Com>) -> anyhow::Result<Ref<Id<GuildMarker>, Guild>> {
            if let Some(guild) = self.guilds.get(&guild_id) {
                return Ok(guild);
            }
            
            let guild = com.http.guild(guild_id).await?.model().await?;
            self.guilds.insert(guild_id, guild);
            
            Ok(self.guilds.get(&guild_id).expect("just inserted"))
        }
        
        pub async fn fetch_guild_channels(&self, guild_id: Id<GuildMarker>, com: Arc<Com>) -> anyhow::Result<Ref<Id<GuildMarker>, Vec<Channel>>> {
            if let Some(channels) = self.guild_channels.get(&guild_id) {
                return Ok(channels);
            }
            
            let channels = com.http.guild_channels(guild_id).await?.model().await?;
            for channel in channels.iter().cloned() {
                self.channels.insert(channel.id, channel);
            }
            self.guild_channels.insert(guild_id, channels);
            
            Ok(self.guild_channels.get(&guild_id).expect("just inserted"))
        }
    }
}
