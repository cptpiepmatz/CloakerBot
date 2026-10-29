use chrono::{DateTime, Local};
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;

use super::Database;
use super::config::GuildConfigRandom;
use super::database::discord_id;

impl Database {
    pub async fn random_configs(
        &self
    ) -> Result<Vec<(Id<GuildMarker>, GuildConfigRandom)>, sqlx::Error> {
        sqlx::query!(
            "SELECT guild_id, random_enabled, random_min_hours, random_max_hours FROM guilds \
             WHERE random_enabled = 1"
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| {
            Ok((discord_id(row.guild_id)?, GuildConfigRandom {
                enabled: Some(true),
                min: row.random_min_hours,
                max: row.random_max_hours
            }))
        })
        .collect()
    }

    /// Claims a due schedule once, or inserts the next schedule if none exists.
    pub async fn claim_random(
        &self,
        guild_id: Id<GuildMarker>,
        config: &GuildConfigRandom,
        now: DateTime<Local>,
        next: DateTime<Local>
    ) -> Result<bool, sqlx::Error> {
        let guild = guild_id.to_string();
        let mut transaction = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let current = sqlx::query!(
            "SELECT random_enabled, random_min_hours, random_max_hours FROM guilds WHERE guild_id \
             = ?",
            guild
        )
        .fetch_optional(&mut *transaction)
        .await?;
        if !current.is_some_and(|row| {
            row.random_enabled == Some(1) &&
                row.random_min_hours == config.min &&
                row.random_max_hours == config.max
        }) {
            return Ok(false);
        }
        let now = now.timestamp_millis();
        let next = next.timestamp_millis();
        let claimed = sqlx::query!(
            "DELETE FROM random_schedules WHERE guild_id = ? AND scheduled_at <= ?",
            guild,
            now
        )
        .execute(&mut *transaction)
        .await?
        .rows_affected() !=
            0;
        if !claimed {
            sqlx::query!(
                "INSERT OR IGNORE INTO random_schedules (guild_id, scheduled_at) VALUES (?, ?)",
                guild,
                next
            )
            .execute(&mut *transaction)
            .await?;
        }
        transaction.commit().await?;
        Ok(claimed)
    }
}
