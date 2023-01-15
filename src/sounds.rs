use std::error::Error;
use std::fmt::{Display, Formatter};

use hyper::client::HttpConnector;
use hyper::{Body, Client as HyperClient, Request};
use twilight_lavalink::http::{LoadedTracks, Track};

use crate::assets::sounds;
use crate::{Com, LAVALINK_ADDR, LAVALINK_AUTH};

type TrackString = String;

#[derive(Debug)]
pub struct LoadedSounds {
    fails: Vec<TrackString>,
    takedowns: Vec<TrackString>,
    wululululu: TrackString
}

// designed after example https://github.com/twilight-rs/twilight/blob/main/examples/lavalink-basic-bot.rs
pub async fn load_sounds() -> anyhow::Result<LoadedSounds> {
    let client = HyperClient::new();

    let mut fails = Vec::with_capacity(sounds::fails::ALL.len());
    for (path, _) in sounds::fails::ALL {
        let path = format!("./sounds/{}", path);
        fails.push(request_track(&client, path).await?);
    }

    let mut takedowns = Vec::with_capacity(sounds::takedowns::ALL.len());
    for (path, _) in sounds::takedowns::ALL {
        let path = format!("./sounds/{}", path);
        takedowns.push(request_track(&client, path).await?)
    }

    let wululululu = request_track(&client, format!("./sounds/{}", sounds::WULULULULU.0)).await?;
    // let (parts, body) = twilight_lavalink::http::load_track(
    //     *LAVALINK_ADDR,
    //     "./sounds/WULULULULU.wav",
    //     LAVALINK_AUTH
    // )?
    // .into_parts();
    // let req = Request::from_parts(parts, Body::from(body));
    // let res = client.request(req).await?;
    // let res_bytes = hyper::body::to_bytes(res.into_body()).await?;
    // let loaded = serde_json::from_slice::<LoadedTracks>(&res_bytes)?;
    // println!("{loaded:?}");

    Ok(LoadedSounds {
        fails,
        takedowns,
        wululululu
    })
}

#[derive(Debug)]
pub struct MatchTrackError {
    path: String
}

impl Display for MatchTrackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "no match for track {}", self.path)
    }
}

impl Error for MatchTrackError {}

async fn request_track(
    client: &HyperClient<HttpConnector>,
    path: String
) -> anyhow::Result<TrackString> {
    let (parts, body) =
        twilight_lavalink::http::load_track(*LAVALINK_ADDR, &path, LAVALINK_AUTH)?.into_parts();
    let req = Request::from_parts(parts, Body::from(body));
    let res = client.request(req).await?;
    let res_bytes = hyper::body::to_bytes(res.into_body()).await?;
    let loaded = serde_json::from_slice::<LoadedTracks>(&res_bytes)?;
    let track = loaded.tracks.first().ok_or(MatchTrackError { path })?;
    Ok(track.track.clone())
}
