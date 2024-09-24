use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::{Future, IntoFuture};

use derive_more::From;
use serde::de::DeserializeOwned;
use twilight_http::response::DeserializeBodyError;
use twilight_http::Response;

pub mod cache;
pub mod log;
pub mod ref_access_map;
pub mod template;
pub mod trim;

#[derive(Debug, From)]
pub enum ClientError {
    Http(twilight_http::Error),
    DeserializeBody(DeserializeBodyError)
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: print somewhere that this is a client error
        match self {
            ClientError::Http(err) => err.fmt(f),
            ClientError::DeserializeBody(err) => err.fmt(f)
        }
    }
}

impl Error for ClientError {}

pub async fn cache_or_fetch<CR, CT, FF, FR, FT, T>(
    cache_res: Option<CR>,
    cache_transformer: CT,
    fetch_future: FF,
    fetch_transformer: FT
) -> Result<T, ClientError>
where
    CT: FnOnce(CR) -> T,
    FF: IntoFuture<Output = Result<Response<FR>, twilight_http::Error>>,
    FR: DeserializeOwned + Unpin,
    FT: FnOnce(FR) -> T
{
    if let Some(cached_data) = cache_res {
        return Ok(cache_transformer(cached_data));
    }

    let response = match fetch_future.await {
        Ok(response) => response,
        Err(err) => return Err(err.into())
    };

    match response.model().await {
        Ok(fetched_data) => Ok(fetch_transformer(fetched_data)),
        Err(err) => Err(err.into())
    }
}

macro_rules! require {
    (Some($unwrap:expr)$( => $ret:expr)?) => {
        match $unwrap {
            Some(inner) => inner,
            None => return $($ret)?,
        }
    };

    ($cond:expr$( => $ret:expr)?) => {
        if !($cond) {
            return $($ret)?;
        }
    };
}

pub(crate) use require;
