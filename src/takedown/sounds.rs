#![allow(deprecated)]

use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::ops::Deref;
use std::sync::LazyLock;
use std::time::Duration;
use std::{io, mem};

use futures::future::try_join_all;
use futures::stream::FuturesUnordered;
use songbird::Call;
use songbird::input::cached::{CodecCacheError, Decompressed, Memory};
use songbird::input::{AudioStream, Input, LiveInput};
use tokio::sync::MutexGuard;
use wav::BitDepth;

use crate::assets;
use crate::panic::ExpectReport;

pub static WULULULULU: LazyLock<Sound> = lazy_sound!(WULULULULU);
pub static FAILS: LazyLock<Vec<Sound>> = lazy_sound!(FAILS+);
pub static TAKEDOWNS: LazyLock<Vec<Sound>> = lazy_sound!(TAKEDOWNS+);

pub struct Sound {
    /// The size of the raw bytes containing that sound. Used for debugging.
    pub raw_size: usize,
    /// The songbird needed input, this implements [`Into<Input>`].
    pub input: Decompressed,
    pub duration: Duration
}

impl Sound {
    fn from_static_wav_bytes(wav_bytes: &'static [u8]) -> Result<Self, SoundFromBytesError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let raw_size = wav_bytes.len();
                let input = Decompressed::new(wav_bytes.into()).await?;
                let duration = wav::read(&mut Cursor::new(wav_bytes))?.calc_duration();
                Ok(Sound { raw_size, input, duration })
            })
        })
    }

    pub async fn play_and_wait(
        &self,
        call: &mut MutexGuard<'_, Call>,
        extra_wait_time: impl Into<Option<Duration>>
    ) {
        call.play_only_input(self.input.new_handle().into());
        tokio::time::sleep(self.duration + extra_wait_time.into().unwrap_or_default()).await;
    }

    pub fn load_eagerly() {
        WULULULULU.deref();
        FAILS.deref();
        TAKEDOWNS.deref();
    }
}

impl Debug for Sound {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let kb = self.raw_size as f32 / 1000.0;
        match f.alternate() {
            false => write!(
                f,
                concat!(
                    "Sound {{",
                    " raw_size: {kb:.1} KB,",
                    " input: Decompressed(...),",
                    " duration: {duration:?}",
                    " }}"
                ),
                kb = kb,
                duration = self.duration
            ),

            true => write!(
                f,
                concat!(
                    "Sound {{\n",
                    "    raw_size: {kb:.1} KB,\n",
                    "    input: Decompressed(...),\n",
                    "    duration: {duration:?},\n",
                    "}}"
                ),
                kb = kb,
                duration = self.duration
            )
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SoundFromBytesError {
    #[error("could not cache decoded sound")]
    CodecCache(#[from] CodecCacheError),

    #[error("could not read WAV sound")]
    Io(#[from] io::Error)
}

macro_rules! lazy_sound {
    ($ident:ident) => {
        LazyLock::new(|| {
            Sound::from_static_wav_bytes(assets::sounds::$ident).expect_report(concat!(
                "could not load ",
                stringify!($ident),
                " sound"
            ))
        })
    };

    ($ident:ident+) => {
        LazyLock::new(|| {
            assets::sounds::$ident
                .into_iter()
                .enumerate()
                .map(|(i, bytes)| {
                    Sound::from_static_wav_bytes(bytes)
                        .expect_report(format!("could not load {}[{i}] sound", stringify!($ident)))
                })
                .collect()
        })
    };
}

use lazy_sound;

trait CalcDuration {
    fn calc_duration(&self) -> Duration;
}

impl CalcDuration for (wav::Header, wav::BitDepth) {
    fn calc_duration(&self) -> Duration {
        let samples = match &self.1 {
            BitDepth::Eight(samples) => samples.len(),
            BitDepth::Sixteen(samples) => samples.len(),
            BitDepth::TwentyFour(samples) => samples.len(),
            BitDepth::ThirtyTwoFloat(samples) => samples.len(),
            BitDepth::Empty => 0
        };

        Duration::from_secs_f64(
            samples as f64 / f64::from(self.0.channel_count) / f64::from(self.0.sampling_rate)
        )
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::io::Cursor;

    use crate::assets;
    use crate::takedown::sounds::CalcDuration;

    #[test]
    fn songbird_uses_same_symphonia_core() {
        assert_eq!(
            TypeId::of::<symphonia::core::codecs::CodecRegistry>(),
            TypeId::of::<songbird::input::core::codecs::CodecRegistry>(),
            "Symphonia must match Songbird's version so audio features apply to its decoder"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn bundled_sounds_load() {
        super::Sound::load_eagerly();
    }

    #[test]
    fn calc_duration_works() {
        let duration =
            wav::read(&mut Cursor::new(assets::sounds::WULULULULU)).unwrap().calc_duration();
        assert_eq!(duration, std::time::Duration::from_nanos(5_217_777_778));
    }
}
