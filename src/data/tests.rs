use chrono::{Duration, Local, Utc};
use twilight_model::id::Id;

use super::Database;
use super::linked_roles::{SecretToken, StoredTokens};
use super::owner_notifications::OwnerNotified;
use super::stats::StatEvent;

#[tokio::test]
async fn construction_fails_when_database_directory_is_a_file() {
    let directory = tempfile::tempdir().unwrap();
    let parent = directory.path().join("not-a-directory");
    tokio::fs::write(&parent, b"").await.unwrap();

    assert!(Database::initialize(parent.join("cloaker.sqlite")).await.is_err());
}

#[tokio::test]
async fn standalone_backup_and_failed_publication() {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::initialize(directory.path().join("cloaker.sqlite")).await.unwrap();
    let guild = Id::new(1);
    let caller = Id::new(2);
    let event = StatEvent::Takedown { caller_id: caller, target_id: Id::new(3) };
    database.record_stat(guild, event).await.unwrap();
    database.backup().await.unwrap();
    let snapshot = database.backup_path();
    let previous = tokio::fs::read(&snapshot).await.unwrap();
    let restored_path = directory.path().join("restored.sqlite");
    tokio::fs::copy(&snapshot, &restored_path).await.unwrap();

    database.record_stat(guild, event).await.unwrap();
    let temporary = snapshot.with_extension("sqlite.tmp");
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;

        let reader = std::fs::OpenOptions::new().read(true).share_mode(1).open(&snapshot).unwrap();
        assert!(database.backup().await.is_err());
        assert_eq!(tokio::fs::read(&snapshot).await.unwrap(), previous);
        drop(reader);
        tokio::fs::remove_file(&temporary).await.unwrap();
    }
    tokio::fs::create_dir(&temporary).await.unwrap();
    assert!(database.backup().await.is_err());
    assert_eq!(tokio::fs::read(&snapshot).await.unwrap(), previous);
    tokio::fs::remove_dir(&temporary).await.unwrap();
    database.backup().await.unwrap();
    assert!(!temporary.exists());

    let restored = Database::initialize(&restored_path).await.unwrap();
    assert_eq!(restored.user_totals(caller).await.unwrap().initiated, 1);
    let integrity: String =
        sqlx::query_scalar("PRAGMA integrity_check").fetch_one(&restored.pool).await.unwrap();
    assert_eq!(integrity, "ok");
    restored.pool.close().await;

    let latest_path = directory.path().join("latest.sqlite");
    tokio::fs::copy(&snapshot, &latest_path).await.unwrap();
    let latest = Database::initialize(latest_path).await.unwrap();
    assert_eq!(latest.user_totals(caller).await.unwrap().initiated, 2);
    latest.pool.close().await;
    database.pool.close().await;
}

#[tokio::test]
async fn storage_roundtrip_and_guild_cleanup() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("private/cloaker.sqlite");
    let database = Database::initialize(&path).await.unwrap();

    let guild = Id::new(u64::MAX);
    let other_guild = Id::new(2);
    let caller = Id::new(u64::MAX);
    let target = Id::new(3);
    let channel = Id::new(u64::MAX);
    let role = Id::new(u64::MAX);
    assert!(database.guild_config(guild).await.unwrap().danger_zones.is_empty());
    let config = database
        .update_config(guild, |config| {
            config.log_channel = Some(channel);
            config.danger_zones = vec![channel];
            config.immunity.general_silent = vec![role];
            config.random.enabled = Some(true);
            config.random.min = Some(2.0);
            config.random.max = Some(3.0);
        })
        .await
        .unwrap();
    assert!(
        database
            .update_config(guild, |config| {
                config.log_channel = None;
                config.random.min = Some(4.0);
            })
            .await
            .is_err()
    );
    assert_eq!(database.guild_config(guild).await.unwrap().log_channel, Some(channel));

    database
        .record_stat(guild, StatEvent::Takedown { caller_id: caller, target_id: target })
        .await
        .unwrap();
    database
        .record_stat(other_guild, StatEvent::Takedown { caller_id: caller, target_id: target })
        .await
        .unwrap();
    database
        .record_stat(guild, StatEvent::Counter { caller_id: target, target_id: caller })
        .await
        .unwrap();
    database.record_stat(guild, StatEvent::Parry { user_id: target }).await.unwrap();
    let totals = database.user_totals(caller).await.unwrap();
    assert_eq!((totals.initiated, totals.received, totals.countered), (2, 1, 0));
    let totals = database.user_totals(target).await.unwrap();
    assert_eq!((totals.initiated, totals.received, totals.countered), (0, 2, 2));
    let snapshot = database.stats().await.unwrap();
    assert_eq!(snapshot.takedowns.punisher_ranks().collect::<Vec<_>>(), vec![(1, (caller, 2))]);
    assert_eq!(snapshot.takedowns.counter_ranks().collect::<Vec<_>>(), vec![(1, (target, 2))]);

    let now = Local::now();
    let next = now + Duration::hours(2);
    assert!(!database.claim_random(guild, &config.random, now, next).await.unwrap());
    assert!(
        database
            .claim_random(guild, &config.random, next, next + Duration::hours(2))
            .await
            .unwrap()
    );
    assert!(
        !database
            .claim_random(guild, &config.random, next, next + Duration::hours(2))
            .await
            .unwrap()
    );
    let disabled =
        database.update_config(guild, |config| config.random.enabled = Some(false)).await.unwrap();
    assert!(
        !database
            .claim_random(
                guild,
                &config.random,
                next + Duration::hours(3),
                next + Duration::hours(5)
            )
            .await
            .unwrap()
    );
    assert!(database.random_configs().await.unwrap().is_empty());
    assert_eq!(disabled.random.enabled, Some(false));

    assert_eq!(database.owner_notification(guild, caller).await.unwrap().allow_dms, None);
    database
        .save_owner_notification(caller, &OwnerNotified {
            allow_dms: Some(false),
            latest_msg_index: 1
        })
        .await
        .unwrap();
    assert_eq!(
        database.owner_notification(other_guild, caller).await.unwrap().allow_dms,
        Some(false)
    );
    let mut tokens = StoredTokens {
        access_token: SecretToken::try_from("access".to_owned()).unwrap(),
        refresh_token: SecretToken::try_from("refresh".to_owned()).unwrap(),
        expires_at: Utc::now() + Duration::hours(1)
    };
    database.save_credentials(caller, &tokens).await.unwrap();
    tokens.refresh_token = SecretToken::try_from("rotated".to_owned()).unwrap();
    database.save_credentials(caller, &tokens).await.unwrap();
    database.pool.close().await;

    let database = Database::initialize(&path).await.unwrap();
    let config = database.guild_config(guild).await.unwrap();
    assert_eq!(config.danger_zones, vec![channel]);
    assert_eq!(config.immunity.general_silent, vec![role]);
    assert_eq!(
        database.credentials(caller).await.unwrap().unwrap().refresh_token.expose(),
        "rotated"
    );
    assert_eq!(database.linked_users().await.unwrap(), vec![caller]);
    assert_eq!(database.user_totals(caller).await.unwrap().initiated, 2);
    database.remove_guild(guild).await.unwrap();
    assert_eq!(database.user_totals(caller).await.unwrap().initiated, 1);
    assert_eq!(database.user_totals(target).await.unwrap().countered, 0);
    assert!(database.guild_config(guild).await.unwrap().danger_zones.is_empty());
    assert_eq!(database.owner_notification(other_guild, caller).await.unwrap().latest_msg_index, 1);
    database.remove_guild(other_guild).await.unwrap();
    assert_eq!(database.user_totals(caller).await.unwrap().initiated, 0);
    assert!(database.credentials(caller).await.unwrap().is_some());
    database.remove_credentials(caller).await.unwrap();
    assert!(database.linked_users().await.unwrap().is_empty());

    let journal: String =
        sqlx::query_scalar("PRAGMA journal_mode").fetch_one(&database.pool).await.unwrap();
    let synchronous: i64 =
        sqlx::query_scalar("PRAGMA synchronous").fetch_one(&database.pool).await.unwrap();
    let foreign_keys: i64 =
        sqlx::query_scalar("PRAGMA foreign_keys").fetch_one(&database.pool).await.unwrap();
    assert_eq!((journal.as_str(), synchronous, foreign_keys), ("wal", 2, 1));
    database.pool.close().await;
}

#[tokio::test]
async fn concurrent_updates_and_schema_constraints() {
    let directory = tempfile::tempdir().unwrap();
    let database = Database::initialize(directory.path().join("cloaker.sqlite")).await.unwrap();
    let guild = Id::new(1);
    let caller = Id::new(2);
    let target = Id::new(3);
    let increments = (0..32).map(|_| {
        database.record_stat(guild, StatEvent::Takedown { caller_id: caller, target_id: target })
    });
    for result in futures::future::join_all(increments).await {
        result.unwrap();
    }
    assert_eq!(database.user_totals(caller).await.unwrap().initiated, 32);
    let (first, second) = tokio::join!(
        database.update_config(guild, |config| config.log_channel = Some(Id::new(4))),
        database.update_config(guild, |config| config.danger_zones = vec![Id::new(5)])
    );
    first.unwrap();
    second.unwrap();
    let config = database.guild_config(guild).await.unwrap();
    assert_eq!(config.log_channel, Some(Id::new(4)));
    assert_eq!(config.danger_zones, vec![Id::new(5)]);
    assert!(
        sqlx::query("UPDATE takedown_counts SET takedowns = -1")
            .execute(&database.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("INSERT INTO danger_zones VALUES ('missing', '5')")
            .execute(&database.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE guilds SET random_min_hours = 0")
            .execute(&database.pool)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE guilds SET random_min_hours = 25")
            .execute(&database.pool)
            .await
            .is_err()
    );
    assert_eq!(database.user_totals(caller).await.unwrap().initiated, 32);
    database.pool.close().await;
}
