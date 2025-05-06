use derive_more::From;
use serde_json::{Value, json};
use twilight_http::{
    request::{Method, RequestBuilder},
    routing::Path,
};

use crate::{APPLICATION_ID, STATE};

// TODO: replace this when application role connection support lands

/// Construct the Application Role Connection Metadata Object list.
///
/// This is sent to discord to define what metadata we provide.
fn metadata() -> Value {
    json!([
        {
            "type": 2, // INTEGER_GREATER_THAN_OR_EQUAL,
            "key": "times_smacked",
            "name": "Time Smacked",
            "description": "Number of successful takedowns initiated."
        },
        {
            "type": 2,
            "key": "times_called",
            "name": "Times Called",
            "description": "Number of times the user was taken down."
        },
        {
            "type": 2,
            "key": "times_countered",
            "name": "Times Countered",
            "description": "Number of successful counters and parries."
        }
    ])
}

#[derive(Debug, From)]
pub enum RegisterMetadataRecordsError {
    Http(twilight_http::Error),
}

pub async fn register_metadata_records() -> Result<(), RegisterMetadataRecordsError> {
    let request = RequestBuilder::raw(
        Method::Put,
        Path::ApplicationIdRoleConnections(APPLICATION_ID.into()),
        format!("applications/{}/role-connections/metadata", APPLICATION_ID),
    )
    .json(&metadata())
    .build()?;

    STATE.client.request::<Value>(request).await?;

    Ok(())
}
