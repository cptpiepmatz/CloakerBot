use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::{Future, IntoFuture};

use derive_more::From;
use serde::de::DeserializeOwned;
use thiserror::Error;
use twilight_http::Response;
use twilight_http::response::DeserializeBodyError;

pub mod cache;
pub mod component_template;
pub mod iterator_ext;
pub mod ref_access_map;
pub mod template_format;
pub mod trim;

#[derive(Debug, Error)]
#[error("client error")]
pub enum ClientError {
    Http(#[from] twilight_http::Error),
    DeserializeBody(#[from] DeserializeBodyError)
}

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

    (Ok($unwrap:expr)$( => $ret:expr)?) => {
        match $unwrap {
            Ok(inner) => inner,
            Err(_) => return $($ret)?,
        }
    };

    ($cond:expr$( => $ret:expr)?) => {
        if !($cond) {
            return $($ret)?;
        }
    };
}

pub(crate) use require;

macro_rules! map {
    ($($key:tt$(: $value:expr)?),* $(,)?) => {
        std::collections::HashMap::from_iter([
            $(map!(@entry: $key $(: $value)?)),*
        ])
    };

    (@entry: $key:ident) => {(std::stringify!($key), $key)};
    (@entry: $key:ident: $value:expr) => {(std::stringify!($key), $value)};
    (@entry: $key:literal: $value:expr) => {($key, $value)};
}

pub(crate) use map;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn expected<K: Eq + std::hash::Hash, V>(pairs: Vec<(K, V)>) -> HashMap<K, V> {
        HashMap::from_iter(pairs)
    }

    #[test]
    fn test_ident_key_same_value() {
        let foo = 123;
        let bar = 456;

        let result = map! {foo, bar};
        let expected = expected(vec![("foo", 123), ("bar", 456)]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_ident_key_custom_value() {
        let result = map! {
            foo: 1,
            bar: 2
        };
        let expected = expected(vec![("foo", 1), ("bar", 2)]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_literal_key() {
        let result = map! {
            "a": 10,
            "b": 20
        };
        let expected = expected(vec![("a", 10), ("b", 20)]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_keys() {
        let x = 7;
        let result = map! {
            x,
            y: 9,
            "z": 11
        };
        let expected = expected(vec![("x", 7), ("y", 9), ("z", 11)]);

        assert_eq!(result, expected);
    }
}
