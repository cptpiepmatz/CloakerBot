use derive_more::From;
use serde::Serialize;
use twilight_http::Client;
use twilight_http::request::{Method, RequestBuilder};
use twilight_http::routing::Path;

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
        Path::ApplicationIdRoleConnections(APPLICATION_ID.into()),
        format!("applications/{}/role-connections/metadata", APPLICATION_ID)
    )
    .json(&metadata())
    .build()?;

    STATE.client.request::<serde_json::Value>(request).await?;

    Ok(())
}

pub enum UpdateMetadataError {}

pub async fn update_metadata(client: &Client) {
    let request = RequestBuilder::raw(
        Method::Put,
        Path::UsersMeApplicationsIdRoleConnection,
        format!("users/@me/applications/{}/role-connection", APPLICATION_ID)
    );

    // TODO: the rest, I guess
}
