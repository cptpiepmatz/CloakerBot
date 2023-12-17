use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::time::Duration;
use std::{io, mem};

use derive_more::From;
use futures::future::try_join_all;
use futures::stream::FuturesUnordered;
use songbird::input::cached::{CodecCacheError, Decompressed, Memory};
use songbird::input::{AudioStream, Input, LiveInput};
use songbird::Call;
use tokio::sync::MutexGuard;
use wav::BitDepth;

use crate::assets;
use crate::sync::OnceLockExpect;

pub static WULULULULU: OnceLockExpect<Sound> = OnceLockExpect::new();
pub static FAILS: OnceLockExpect<Vec<Sound>> = OnceLockExpect::new();
pub static TAKEDOWNS: OnceLockExpect<Vec<Sound>> = OnceLockExpect::new();

pub struct Sound {
    /// The size of the raw bytes containing that sound. Used for debugging.
    pub raw_size: usize,
    /// The songbird needed input, this implements [`Into<Input>`].
    pub input: Decompressed,
    pub duration: Duration
}

impl Sound {
    async fn from_static_wav_bytes(wav_bytes: &'static [u8]) -> Result<Self, SoundFromBytesError> {
        let raw_size = wav_bytes.len();
        let input = Decompressed::new(wav_bytes.into()).await?;
        let duration = wav::read(&mut Cursor::new(wav_bytes))?.calc_duration();
        Ok(Sound {
            raw_size,
            input,
            duration
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

#[derive(Debug, From)]
pub enum SoundFromBytesError {
    CodecCache(CodecCacheError),
    Io(io::Error)
}

pub async fn prep_sounds() -> Result<(), SoundFromBytesError> {
    OnceLockExpect::set(
        &WULULULULU,
        Sound::from_static_wav_bytes(assets::sounds::WULULULULU).await?
    );

    OnceLockExpect::set(
        &FAILS,
        try_join_all(
            assets::sounds::FAILS
                .into_iter()
                .map(|bytes| Sound::from_static_wav_bytes(bytes))
        )
        .await?
    );

    OnceLockExpect::set(
        &TAKEDOWNS,
        try_join_all(
            assets::sounds::TAKEDOWNS
                .into_iter()
                .map(|bytes| Sound::from_static_wav_bytes(bytes))
        )
        .await?
    );

    Ok(())
}

trait CalcDuration {
    fn calc_duration(&self) -> Duration;
}

impl CalcDuration for (wav::Header, wav::BitDepth) {
    fn calc_duration(&self) -> Duration {
        let bytes_per_second = self.0.bytes_per_second;
        let data_bytes: usize = match &self.1 {
            BitDepth::Eight(samples) => samples.len() * mem::size_of::<u8>(),
            BitDepth::Sixteen(samples) => samples.len() * mem::size_of::<i16>(),
            BitDepth::TwentyFour(samples) => samples.len() * mem::size_of::<i32>(),
            BitDepth::ThirtyTwoFloat(samples) => samples.len() * mem::size_of::<f32>(),
            BitDepth::Empty => 0
        };

        Duration::from_secs_f64(data_bytes as f64 / bytes_per_second as f64)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::assets;
    use crate::takedown::sounds::CalcDuration;

    #[test]
    fn calc_duration_works() {
        let duration = wav::read(&mut Cursor::new(assets::sounds::WULULULULU))
            .unwrap()
            .calc_duration();
        // TODO: get a more precise time and check with that, use std::time::Duration
        assert_eq!(duration.as_secs(), 5);
    }
}
