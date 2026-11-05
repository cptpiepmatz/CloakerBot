use sqlx::SqliteConnection;
use twilight_model::id::Id;
use twilight_model::id::marker::GuildMarker;

use super::Database;
use super::config::{CompleteDefault, GuildConfig, GuildConfigRandom};
use super::database::{decode_error, discord_id};

async fn read_config(
    connection: &mut SqliteConnection,
    guild: &str
) -> Result<GuildConfig, sqlx::Error> {
    let mut config = GuildConfig::default();
    config.target_bots = sqlx::query_scalar::<_, bool>(
        "SELECT target_bots FROM guilds WHERE guild_id = ?"
    )
    .bind(guild)
    .fetch_optional(&mut *connection)
    .await?
    .unwrap_or(true);
    if let Some(row) = sqlx::query!(
        "SELECT log_channel_id, random_enabled, random_min_hours, random_max_hours FROM guilds \
         WHERE guild_id = ?",
        guild
    )
    .fetch_optional(&mut *connection)
    .await?
    {
        config.log_channel = row.log_channel_id.map(discord_id).transpose()?;
        config.random = GuildConfigRandom {
            enabled: row.random_enabled.map(|value| value != 0),
            min: row.random_min_hours,
            max: row.random_max_hours
        };
    }
    for row in sqlx::query!(
        "SELECT channel_id FROM danger_zones WHERE guild_id = ? ORDER BY channel_id",
        guild
    )
    .fetch_all(&mut *connection)
    .await?
    {
        config.danger_zones.push(discord_id(row.channel_id)?);
    }
    for row in sqlx::query!(
        "SELECT kind, role_id FROM immunity_roles WHERE guild_id = ? ORDER BY kind, role_id",
        guild
    )
    .fetch_all(&mut *connection)
    .await?
    {
        let roles = match row.kind.as_str() {
            "general" => &mut config.immunity.general,
            "random" => &mut config.immunity.random,
            "general_silent" => &mut config.immunity.general_silent,
            "random_silent" => &mut config.immunity.random_silent,
            _ => return Err(decode_error("invalid immunity kind"))
        };
        roles.push(discord_id(row.role_id)?);
    }
    Ok(config)
}

impl Database {
    pub async fn guild_config(
        &self,
        guild_id: Id<GuildMarker>
    ) -> Result<GuildConfig, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let config = read_config(&mut transaction, &guild_id.to_string()).await?;
        transaction.commit().await?;
        Ok(config)
    }

    /// Applies a configuration edit atomically without a read-modify-write
    /// race.
    pub async fn update_config(
        &self,
        guild_id: Id<GuildMarker>,
        edit: impl FnOnce(&mut GuildConfig)
    ) -> Result<GuildConfig, sqlx::Error> {
        let guild = guild_id.to_string();
        let mut transaction = self.pool.begin_with("BEGIN IMMEDIATE").await?;
        let mut config = read_config(&mut transaction, &guild).await?;
        let previous = config.random.clone();
        edit(&mut config);
        let random = config.random.complete();
        if !random.min.is_finite() ||
            !random.max.is_finite() ||
            random.min <= 0.0 ||
            random.min > random.max ||
            random.max > 8760.0
        {
            return Err(sqlx::Error::InvalidArgument(
                "random intervals must be finite, positive, ordered, and no longer than one year"
                    .into()
            ));
        }
        let log = config.log_channel.map(|id| id.to_string());
        sqlx::query!(
            "INSERT INTO guilds (guild_id, log_channel_id) VALUES (?, ?) ON CONFLICT (guild_id) \
             DO UPDATE SET log_channel_id = excluded.log_channel_id",
            guild,
            log
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query("UPDATE guilds SET target_bots = ? WHERE guild_id = ?")
            .bind(config.target_bots)
            .bind(&guild)
            .execute(&mut *transaction)
            .await?;
        if config.random != previous {
            let enabled = config.random.enabled.map(i64::from);
            sqlx::query!(
                "UPDATE guilds SET random_enabled = ?, random_min_hours = ?, random_max_hours = ? \
                 WHERE guild_id = ?",
                enabled,
                config.random.min,
                config.random.max,
                guild
            )
            .execute(&mut *transaction)
            .await?;
        }
        sqlx::query!("DELETE FROM danger_zones WHERE guild_id = ?", guild)
            .execute(&mut *transaction)
            .await?;
        for channel in &config.danger_zones {
            let channel = channel.to_string();
            sqlx::query!(
                "INSERT OR IGNORE INTO danger_zones (guild_id, channel_id) VALUES (?, ?)",
                guild,
                channel
            )
            .execute(&mut *transaction)
            .await?;
        }
        sqlx::query!("DELETE FROM immunity_roles WHERE guild_id = ?", guild)
            .execute(&mut *transaction)
            .await?;
        for (kind, roles) in [
            ("general", &config.immunity.general),
            ("random", &config.immunity.random),
            ("general_silent", &config.immunity.general_silent),
            ("random_silent", &config.immunity.random_silent)
        ] {
            for role in roles {
                let role = role.to_string();
                sqlx::query!(
                    "INSERT OR IGNORE INTO immunity_roles (guild_id, kind, role_id) VALUES (?, ?, \
                     ?)",
                    guild,
                    kind,
                    role
                )
                .execute(&mut *transaction)
                .await?;
            }
        }
        transaction.commit().await?;
        Ok(config)
    }
}
