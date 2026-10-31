use std::ops::Deref;

use axum::Router;
use axum::response::Redirect;
use axum::routing::get;
use cookie::Key;

use crate::{COOKIE_SECRET, OAUTH_PORT};

pub mod application_role_connection;
pub mod oauth;

pub async fn serve() -> std::io::Result<()> {
    let router = Router::new()
        .route("/", get(async || "👋"))
        .route(oauth::LINKED_ROLE_PATH, get(oauth::linked_role))
        .route(oauth::CALLBACK_PATH, get(oauth::callback))
        .with_state(Key::from(COOKIE_SECRET.as_bytes()));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", OAUTH_PORT)).await?;

    tracing::info!(port = OAUTH_PORT, "HTTP server listening");
    axum::serve(listener, router).await
}
