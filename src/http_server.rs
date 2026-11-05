use std::sync::LazyLock;

use axum::Router;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use http::{HeaderMap, StatusCode, header};
use sha2::{Digest, Sha256};
use twilight_model::guild::Permissions;
use url::Url;

use crate::link::oauth;
use crate::{CLIENT_ID, HTTP_HOST, HTTP_PORT, INVITE_SCOPES};

static_toml::static_toml! {
    static MANIFEST = include_toml!("Cargo.toml");
}

pub async fn serve() -> std::io::Result<()> {
    let router = Router::new().merge(routes()).merge(oauth::routes());
    let listener = tokio::net::TcpListener::bind((HTTP_HOST, HTTP_PORT)).await?;

    tracing::info!(host = HTTP_HOST, port = HTTP_PORT, "http server listening");
    axum::serve(listener, router).await
}

fn routes() -> Router {
    Router::new()
        .route("/", get(async || "👋"))
        .route("/icon.png", get(icon))
        .route("/invite", get(invite))
        .route("/github", get(async || {
            Redirect::temporary(env!("CARGO_PKG_REPOSITORY"))
        }))
        .route("/discord", get(async || {
            Redirect::temporary(MANIFEST.package.metadata.discord_invite)
        }))
        .route("/version", get(async || env!("CARGO_PKG_VERSION")))
}

async fn invite() -> Redirect {
    let permissions = Permissions::VIEW_CHANNEL |
        Permissions::SEND_MESSAGES |
        Permissions::CONNECT |
        Permissions::SPEAK |
        Permissions::MOVE_MEMBERS;
    let mut url = Url::parse("https://discord.com/oauth2/authorize").expect("valid url");
    url.query_pairs_mut()
        .append_pair("client_id", &CLIENT_ID.to_string())
        .append_pair("scope", &INVITE_SCOPES.join(" "))
        .append_pair("permissions", &permissions.bits().to_string())
        .append_pair("integration_type", "0");
    Redirect::temporary(url.as_str())
}

const ICON: &[u8] = include_bytes!("../icon/v3.png");
static ICON_ETAG: LazyLock<String> = LazyLock::new(|| {
    let hash: String = Sha256::digest(ICON).iter().map(|byte| format!("{byte:02x}")).collect();
    format!("\"{hash}\"")
});

async fn icon(headers: HeaderMap) -> Response {
    let not_modified = headers.get_all(header::IF_NONE_MATCH).iter().any(|value| {
        value.to_str().is_ok_and(|value| {
            value.split(',').any(|tag| {
                let tag = tag.trim();
                tag == "*" || tag.strip_prefix("W/").unwrap_or(tag) == ICON_ETAG.as_str()
            })
        })
    });
    let cache_headers =
        [(header::CACHE_CONTROL, "public, no-cache"), (header::ETAG, ICON_ETAG.as_str())];

    if not_modified {
        (StatusCode::NOT_MODIFIED, cache_headers).into_response()
    }
    else {
        (cache_headers, [(header::CONTENT_TYPE, "image/png")], ICON).into_response()
    }
}
