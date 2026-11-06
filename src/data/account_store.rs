use chrono::{DateTime, Utc};
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};

use super::Database;
use super::database::{decode_error, discord_id};
use super::linked_roles::{SecretToken, StoredTokens};
use super::owner_notifications::OwnerNotified;

impl Database {
    pub async fn linked_users(&self) -> Result<Vec<Id<UserMarker>>, sqlx::Error> {
        sqlx::query!("SELECT user_id FROM linked_accounts")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| discord_id(row.user_id))
            .collect()
    }

    pub async fn credentials(
        &self,
        user_id: Id<UserMarker>
    ) -> Result<Option<StoredTokens>, sqlx::Error> {
        let user = user_id.to_string();
        sqlx::query!(
            "SELECT access_token, refresh_token, expires_at FROM linked_accounts WHERE user_id = ?",
            user
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok(StoredTokens {
                access_token: SecretToken::try_from(row.access_token).map_err(decode_error)?,
                refresh_token: SecretToken::try_from(row.refresh_token).map_err(decode_error)?,
                expires_at: DateTime::<Utc>::from_timestamp(row.expires_at, 0)
                    .ok_or_else(|| decode_error("invalid token expiry"))?
            })
        })
        .transpose()
    }

    pub async fn save_credentials(
        &self,
        user_id: Id<UserMarker>,
        tokens: &StoredTokens
    ) -> Result<(), sqlx::Error> {
        let user = user_id.to_string();
        let access = tokens.access_token.expose();
        let refresh = tokens.refresh_token.expose();
        let expires = tokens.expires_at.timestamp();
        sqlx::query!(
            "INSERT INTO linked_accounts (user_id, access_token, refresh_token, expires_at) \
             VALUES (?, ?, ?, ?) ON CONFLICT (user_id) DO UPDATE SET access_token = \
             excluded.access_token, refresh_token = excluded.refresh_token, expires_at = \
             excluded.expires_at",
            user,
            access,
            refresh,
            expires
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_credentials(&self, user_id: Id<UserMarker>) -> Result<(), sqlx::Error> {
        let user = user_id.to_string();
        sqlx::query!("DELETE FROM linked_accounts WHERE user_id = ?", user)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn owner_notification(
        &self,
        user_id: Id<UserMarker>
    ) -> Result<OwnerNotified, sqlx::Error> {
        let user = user_id.to_string();
        let mut transaction = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        sqlx::query!("INSERT OR IGNORE INTO owner_notifications (user_id) VALUES (?)", user)
            .execute(&mut *transaction)
            .await?;
        let row = sqlx::query!(
            "SELECT allow_dms, latest_msg_index FROM owner_notifications WHERE user_id = ?",
            user
        )
        .fetch_one(&mut *transaction)
        .await?;
        transaction.commit().await?;
        Ok(OwnerNotified {
            allow_dms: row.allow_dms.map(|value| value != 0),
            latest_msg_index: row
                .latest_msg_index
                .try_into()
                .map_err(|_| decode_error("invalid announcement index"))?
        })
    }

    pub async fn save_owner_notification(
        &self,
        user_id: Id<UserMarker>,
        notification: &OwnerNotified
    ) -> Result<(), sqlx::Error> {
        let user = user_id.to_string();
        let allow = notification.allow_dms.map(i64::from);
        let index = i64::try_from(notification.latest_msg_index)
            .map_err(|_| decode_error("invalid announcement index"))?;
        sqlx::query!(
            "UPDATE owner_notifications SET allow_dms = ?, latest_msg_index = ? WHERE user_id = ?",
            allow,
            index,
            user
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_owner_notification_allowance(
        &self,
        user_id: Id<UserMarker>,
        allow_dms: bool
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO owner_notifications (user_id, allow_dms) VALUES (?, ?) ON CONFLICT \
             (user_id) DO UPDATE SET allow_dms = excluded.allow_dms"
        )
        .bind(user_id.to_string())
        .bind(allow_dms)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_guild(&self, guild_id: Id<GuildMarker>) -> Result<(), sqlx::Error> {
        let guild = guild_id.to_string();
        sqlx::query!("DELETE FROM guilds WHERE guild_id = ?", guild).execute(&self.pool).await?;
        Ok(())
    }
}
