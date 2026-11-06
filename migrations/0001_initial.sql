CREATE TABLE guilds (
    guild_id TEXT PRIMARY KEY NOT NULL CHECK (guild_id != ''),
    log_channel_id TEXT CHECK (log_channel_id != ''),
    target_bots INTEGER NOT NULL DEFAULT 1 CHECK (target_bots IN (0, 1)),
    random_enabled INTEGER CHECK (random_enabled IN (0, 1)),
    random_min_hours REAL CHECK (random_min_hours > 0 AND random_min_hours <= 8760),
    random_max_hours REAL CHECK (random_max_hours > 0 AND random_max_hours <= 8760),
    CHECK (coalesce(random_min_hours, 1.0) <= coalesce(random_max_hours, 24.0))
) STRICT;

CREATE TABLE danger_zones (
    guild_id TEXT NOT NULL REFERENCES guilds ON DELETE CASCADE,
    channel_id TEXT NOT NULL CHECK (channel_id != ''),
    PRIMARY KEY (guild_id, channel_id)
) STRICT;

CREATE TABLE immunity_roles (
    guild_id TEXT NOT NULL REFERENCES guilds ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('general', 'random', 'general_silent', 'random_silent')),
    role_id TEXT NOT NULL CHECK (role_id != ''),
    PRIMARY KEY (guild_id, kind, role_id)
) STRICT;

CREATE TABLE takedown_counts (
    guild_id TEXT NOT NULL REFERENCES guilds ON DELETE CASCADE,
    caller_id TEXT NOT NULL CHECK (caller_id != ''),
    target_id TEXT NOT NULL CHECK (target_id != ''),
    takedowns INTEGER NOT NULL DEFAULT 0 CHECK (takedowns >= 0),
    counters INTEGER NOT NULL DEFAULT 0 CHECK (counters >= 0),
    PRIMARY KEY (guild_id, caller_id, target_id)
) STRICT;

CREATE INDEX takedown_targets ON takedown_counts (target_id);
CREATE INDEX takedown_callers ON takedown_counts (caller_id);

CREATE TABLE parry_counts (
    guild_id TEXT NOT NULL REFERENCES guilds ON DELETE CASCADE,
    user_id TEXT NOT NULL CHECK (user_id != ''),
    parries INTEGER NOT NULL CHECK (parries >= 0),
    PRIMARY KEY (guild_id, user_id)
) STRICT;

CREATE TABLE random_schedules (
    guild_id TEXT PRIMARY KEY NOT NULL REFERENCES guilds ON DELETE CASCADE,
    scheduled_at INTEGER NOT NULL
) STRICT;

CREATE TRIGGER reset_random_schedule
AFTER UPDATE OF random_enabled, random_min_hours, random_max_hours ON guilds
BEGIN
    DELETE FROM random_schedules WHERE guild_id = new.guild_id;
END;

CREATE TABLE owner_notifications (
    user_id TEXT PRIMARY KEY NOT NULL CHECK (user_id != ''),
    allow_dms INTEGER CHECK (allow_dms IN (0, 1)),
    latest_msg_index INTEGER NOT NULL DEFAULT 0 CHECK (latest_msg_index >= 0)
) STRICT;

CREATE TABLE linked_accounts (
    user_id TEXT PRIMARY KEY NOT NULL CHECK (user_id != ''),
    access_token TEXT NOT NULL CHECK (length(access_token) > 0),
    refresh_token TEXT NOT NULL CHECK (length(refresh_token) > 0),
    expires_at INTEGER NOT NULL
) STRICT;
