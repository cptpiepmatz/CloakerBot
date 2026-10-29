use derive_more::From;
use serde::Serialize;
use twilight_http::Client;
use twilight_http::request::{Method, RequestBuilder};

use crate::{APPLICATION_ID, STATE};

#[derive(Serialize)]
#[repr(u8)]
enum MetadataKind {
    IntegerGreaterThanOrEqual = 2
}

#[derive(Serialize)]
struct MetadataRecord {
    #[serde(rename = "type", serialize_with = "serialize_kind")]
    kind: MetadataKind,
    key: &'static str,
    name: &'static str,
    description: &'static str
}

fn serialize_kind<S: serde::Serializer>(kind: &MetadataKind, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u8(match kind { MetadataKind::IntegerGreaterThanOrEqual => 2 })
}

/// Construct the Application Role Connection Metadata Object list.
///
/// This is sent to discord to define what metadata we provide.
fn metadata() -> [MetadataRecord; 3] {
    [
        MetadataRecord {
            kind: MetadataKind::IntegerGreaterThanOrEqual,
            key: "times_smacked",
            name: "Times Smacked",
            description: "Number of successful takedowns initiated globally."
        },
        MetadataRecord {
            kind: MetadataKind::IntegerGreaterThanOrEqual,
            key: "times_called",
            name: "Times Called",
            description: "Number of times the user was taken down globally."
        },
        MetadataRecord {
            kind: MetadataKind::IntegerGreaterThanOrEqual,
            key: "times_countered",
            name: "Times Countered",
            description: "Number of successful counters and parries globally."
        }
    ]
}

#[derive(Debug, From)]
pub enum RegisterMetadataRecordsError {
    Http(twilight_http::Error)
}

pub async fn register_metadata() -> Result<(), RegisterMetadataRecordsError> {
    let request = RequestBuilder::raw(
        Method::Put,
        format!("applications/{}/role-connections/metadata", APPLICATION_ID)
    )
    .json(&metadata())
    .build()?;

    STATE.client.request::<serde_json::Value>(request).await?;

    Ok(())
}

#[derive(Serialize)]
struct RoleConnection {
    platform_name: &'static str,
    metadata: UserMetadata
}

#[derive(Serialize)]
struct UserMetadata {
    times_smacked: String,
    times_called: String,
    times_countered: String
}

pub async fn update_metadata(
    client: &Client,
    user_id: twilight_model::id::Id<twilight_model::id::marker::UserMarker>
) -> Result<(), twilight_http::Error> {
    let totals = STATE.stats.read().await.takedowns.user_totals(user_id);
    let connection = RoleConnection {
        platform_name: "CloakerBot",
        metadata: UserMetadata {
            times_smacked: totals.initiated.to_string(),
            times_called: totals.received.to_string(),
            times_countered: totals.countered.to_string()
        }
    };
    let request = RequestBuilder::raw(
        Method::Put,
        format!("users/@me/applications/{}/role-connection", APPLICATION_ID)
    ).json(&connection).build()?;
    client.request::<serde_json::Value>(request).await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum SyncError {
    #[error(transparent)]
    Refresh(#[from] super::oauth::RefreshError),
    #[error(transparent)]
    Http(#[from] twilight_http::Error),
    #[error(transparent)]
    Persist(#[from] std::io::Error)
}

async fn refresh_and_store(
    user_id: twilight_model::id::Id<twilight_model::id::marker::UserMarker>,
    tokens: &crate::data::linked_roles::StoredTokens
) -> Result<crate::data::linked_roles::StoredTokens, SyncError> {
    let tokens = match super::oauth::refresh_tokens(&tokens.refresh_token).await {
        Err(super::oauth::RefreshError::Revoked) => {
            STATE.linked_roles.write().await.remove(&user_id);
            STATE.linked_roles.store().await?;
            return Err(super::oauth::RefreshError::Revoked.into());
        }
        result => result?
    };
    STATE.linked_roles.write().await.insert(user_id, tokens.clone());
    STATE.linked_roles.store().await?;
    Ok(tokens)
}

async fn sync_user(user_id: twilight_model::id::Id<twilight_model::id::marker::UserMarker>) -> Result<(), SyncError> {
    let _lock = STATE.linked_role_locks.lock(user_id).await;
    let Some(mut tokens) = STATE.linked_roles.read().await.get(&user_id).cloned() else { return Ok(()); };
    if tokens.expires_at <= chrono::Utc::now() + chrono::Duration::seconds(60) {
        tokens = refresh_and_store(user_id, &tokens).await?;
    }
    let client = Client::new(format!("Bearer {}", tokens.access_token.expose()));
    match update_metadata(&client, user_id).await {
        Err(error) if matches!(error.kind(), twilight_http::error::ErrorType::Response { status, .. } if status.get() == 401) => {
            tokens = refresh_and_store(user_id, &tokens).await?;
            let client = Client::new(format!("Bearer {}", tokens.access_token.expose()));
            update_metadata(&client, user_id).await?;
        }
        result => { result?; }
    }
    Ok(())
}

pub async fn maintain_connections() {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let users: Vec<_> = STATE.linked_roles.read().await.keys().copied().collect();
        for user_id in users {
            match sync_user(user_id).await {
                Err(SyncError::Refresh(super::oauth::RefreshError::Revoked)) => {
                    macros::info!("removed revoked linked-role connection for {user_id}");
                }
                Err(error) => macros::warn!("linked-role sync failed for {user_id}: {error:?}"),
                Ok(()) => {}
            }
        }
    }
}
