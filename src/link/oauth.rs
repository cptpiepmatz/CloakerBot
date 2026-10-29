use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::sync::LazyLock;

use axum::extract::Query;
use axum::response::{Html, IntoResponse, Redirect};
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::Cookie;
use futures::TryFutureExt;
use http::StatusCode;
use rand::distr::{Alphanumeric, SampleString};
use serde::Deserialize;
use twilight_http::Client as TwilightClient;
use url::Url;

use crate::util::map;
use crate::{CLIENT_SECRET, LazyString, REDIRECT_URI, STATE};
use crate::data::linked_roles::{SecretToken, StoredTokens};

static CLIENT_ID: LazyString = LazyLock::new(|| crate::CLIENT_ID.to_string());
static PENDING_STATES: LazyLock<parking_lot::Mutex<HashMap<Nonce, std::time::Instant>>> =
    LazyLock::new(Default::default);
const STATE_LIFETIME: std::time::Duration = std::time::Duration::from_secs(300);

pub const LINKED_ROLE_PATH: &str = "/linked-role";
pub async fn linked_role(jar: SignedCookieJar) -> Result<(SignedCookieJar, Redirect), StatusCode> {
    let (oauth_url, state) = oauth_url();
    let max_age = cookie::time::Duration::minutes(5);
    {
        let mut pending = PENDING_STATES.lock();
        pending.retain(|_, created| created.elapsed() < STATE_LIFETIME);
        if pending.len() >= 4096 { return Err(StatusCode::TOO_MANY_REQUESTS); }
        pending.insert(state.clone(), std::time::Instant::now());
    }
    let jar = jar.add(Cookie::build(("state", state.0))
        .path(CALLBACK_PATH).max_age(max_age).secure(true).http_only(true)
        .same_site(cookie::SameSite::Lax));
    Ok((jar, Redirect::to(oauth_url.as_str())))
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    code: String,
    state: Nonce
}

pub const CALLBACK_PATH: &str = "/callback";
pub async fn callback(
    Query(params): Query<CallbackParams>,
    jar: SignedCookieJar
) -> Result<(SignedCookieJar, Html<&'static str>), StatusCode> {
    fn err500(err: impl Error) -> StatusCode {
        macros::error!("callback error: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    }

    let code = params.code;
    let discord_state = params.state;

    let client_state = jar.get("state").map(|c| Nonce(c.value_trimmed().to_owned()));
    if Some(&discord_state) != client_state.as_ref() {
        macros::warn!("state verification failed");
        return Err(StatusCode::FORBIDDEN);
    }

    if !PENDING_STATES.lock().remove(&discord_state).is_some_and(|created| created.elapsed() < STATE_LIFETIME) {
        return Err(StatusCode::FORBIDDEN);
    }
    let jar = jar.remove(Cookie::build("state").path(CALLBACK_PATH));

    let tokens = oauth_tokens(&code).await.map_err(err500)?;

    let client = TwilightClient::new(format!("Bearer {}", tokens.access_token.expose()));
    let me_data =
        client.current_authorization().await.map_err(err500)?.model().await.map_err(err500)?;

    let me_user = match me_data.user {
        None => return Err(StatusCode::FORBIDDEN),
        Some(user) => user
    };

    let tokens = tokens.into_stored().map_err(err500)?;
    let _lock = STATE.linked_role_locks.lock(me_user.id).await;
    STATE.linked_roles.write().await.insert(me_user.id, tokens);
    STATE.linked_roles.store().await.map_err(err500)?;
    super::application_role_connection::update_metadata(&client, me_user.id).await.map_err(err500)?;

    if let Err(error) = notify_connected(me_user.id).await {
        macros::warn!("linked-role confirmation DM failed: {error:?}");
    }

    Ok((jar, Html("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>CloakerBot connected</title><h1>Account connected 🔗</h1><p>Your global takedown, victim, and counter stats are now available for Discord linked roles and will update automatically.</p><p>You can close this page or <a href=\"https://discord.com/channels/@me/\">return to Discord</a>.</p></html>")))
}

async fn notify_connected(user_id: twilight_model::id::Id<twilight_model::id::marker::UserMarker>) -> Result<(), crate::util::ClientError> {
    let private_channel = STATE
        .client
        .create_private_channel(user_id)
        .await
        ?
        .model()
        .await
        ?;
    STATE
        .client
        .create_message(private_channel.id)
        .content("## 🔗 Account connected\nYour global takedown, victim, and counter stats are now available for Discord linked roles. Stats update automatically; server admins choose the role requirements. You can disconnect CloakerBot in Discord's User Settings → Authorized Apps.")
        .await
        ?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Nonce(pub String);

fn oauth_url() -> (Url, Nonce) {
    let state = Alphanumeric.sample_string(&mut rand::rng(), 25);

    let mut url = Url::parse("https://discord.com/api/oauth2/authorize").expect("valid url");
    url.query_pairs_mut()
        .append_pair("client_id", &*CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("state", &state)
        .append_pair("scope", "identify role_connections.write")
        .append_pair("prompt", "consent");

    (url, Nonce(state))
}

#[derive(Debug, Deserialize)]
enum Bearer {
    #[serde(rename = "Bearer", alias = "bearer")]
    Bearer
}

#[derive(Debug, thiserror::Error)]
#[error("Discord did not grant identify and role_connections.write")]
pub struct MissingScopes;

#[derive(Debug, Deserialize)]
pub struct Tokens {
    access_token: SecretToken,
    token_type: Bearer,
    expires_in: std::num::NonZeroU32,
    refresh_token: SecretToken,
    scope: String
}

impl Tokens {
    pub fn into_stored(self) -> Result<StoredTokens, MissingScopes> {
        if !["identify", "role_connections.write"].iter()
            .all(|required| self.scope.split_whitespace().any(|scope| scope == *required)) {
            return Err(MissingScopes);
        }
        Ok(StoredTokens {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(i64::from(self.expires_in.get()))
        })
    }
}

async fn oauth_tokens(code: &str) -> reqwest::Result<Tokens> {
    let url = "https://discord.com/api/v10/oauth2/token";
    let body: HashMap<_, &str> = map! {
        client_id: CLIENT_ID.as_str(),
        client_secret: CLIENT_SECRET.as_str(),
        grant_type: "authorization_code",
        code,
        redirect_uri: REDIRECT_URI,
    };

    STATE
        .http_client
        .post(url)
        .form(&body)
        .header(reqwest::header::CONTENT_TYPE, mime::APPLICATION_WWW_FORM_URLENCODED.essence_str())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error("OAuth token request failed")]
    Request(#[from] reqwest::Error),
    #[error("OAuth authorization was revoked")]
    Revoked,
    #[error(transparent)]
    MissingScopes(#[from] MissingScopes)
}

pub async fn refresh_tokens(refresh_token: &SecretToken) -> Result<StoredTokens, RefreshError> {
    let body: HashMap<_, &str> = map! {
            client_id: CLIENT_ID.as_str(),
            client_secret: CLIENT_SECRET.as_str(),
            grant_type: "refresh_token",
            refresh_token: refresh_token.expose()
        };
    let response = STATE.http_client.post("https://discord.com/api/v10/oauth2/token")
        .form(&body).send().await?;
    if response.status() == StatusCode::BAD_REQUEST {
        let error = response.error_for_status_ref().expect_err("400 response");
        #[derive(Deserialize)]
        struct OAuthError { error: String }
        return match response.json::<OAuthError>().await {
            Ok(body) if body.error == "invalid_grant" => Err(RefreshError::Revoked),
            _ => Err(RefreshError::Request(error))
        };
    }
    Ok(response.error_for_status()?.json::<Tokens>().await?.into_stored()?)
}
