use std::ops::Deref;
use std::sync::LazyLock;

use axum::Router;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use cookie::Key;
use http::{HeaderMap, StatusCode, header};
use sha2::{Digest, Sha256};

use crate::{COOKIE_SECRET, OAUTH_PORT};

pub mod application_role_connection;
pub mod oauth;

pub async fn serve() -> std::io::Result<()> {
    let router = Router::new()
        .route("/", get(async || "👋"))
        .route("/icon.png", get(icon))
        .route(oauth::LINKED_ROLE_PATH, get(oauth::linked_role))
        .route(oauth::CALLBACK_PATH, get(oauth::callback))
        .with_state(Key::from(COOKIE_SECRET.as_bytes()));

    #[cfg(debug_assertions)]
    let router = router.route("/debug/connected", get(async || connected_page()));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", OAUTH_PORT)).await?;

    tracing::info!(port = OAUTH_PORT, "http server listening");
    axum::serve(listener, router).await
}

fn connected_page() -> Response {
    (
        [(header::CACHE_CONTROL, "no-cache, no-store")],
        Html(include_str!("../../assets/oauth/connected.html"))
    )
        .into_response()
}

const ICON: &[u8] = include_bytes!("../../icon/v3.png");
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
