use std::io::Cursor;
use std::time::Duration;

use lazy_static::lazy_static;
use rand::Rng;
use songbird::input::cached::Decompressed;
use songbird::input::Input;
use symphonia::core::codecs::CodecRegistry;
use symphonia::core::probe::Probe;
use wav::Header;

use crate::assets;

lazy_static! {
    static ref CODECS: &'static CodecRegistry = symphonia::default::get_codecs();
    static ref PROBE: &'static Probe = symphonia::default::get_probe();
}

pub struct Sound {
    pub data: Decompressed,
    pub duration: Duration
}

pub struct Sounds {
    pub wululululu: Sound,
    pub takedowns: Vec<Sound>,
    pub fails: Vec<Sound>
}

impl Sound {
    async fn make(bytes: &'static [u8]) -> anyhow::Result<Sound> {
        let data = Decompressed::new(Input::from(bytes)).await?;
        let (
            Header {
                channel_count,
                bytes_per_second,
                ..
            },
            _
        ) = wav::read(&mut Cursor::new(bytes))?;
        let channel_count = channel_count as f64;
        let bytes_per_second = bytes_per_second as f64;
        let file_bytes = bytes.len() as f64;
        let duration = Duration::from_secs_f64(file_bytes / (channel_count * bytes_per_second));
        Ok(Sound { data, duration })
    }
}

impl Sounds {
    pub async fn prepare() -> anyhow::Result<Self> {
        let wululululu = Sound::make(assets::sounds::WULULULULU).await?;

        let mut takedowns = Vec::with_capacity(assets::sounds::takedowns::ALL.len());
        for sound in assets::sounds::takedowns::ALL {
            takedowns.push(Sound::make(sound).await?);
        }

        let mut fails = Vec::with_capacity(assets::sounds::fails::ALL.len());
        for sound in assets::sounds::fails::ALL {
            fails.push(Sound::make(sound).await?);
        }

        Ok(Sounds {
            wululululu,
            takedowns,
            fails
        })
    }

    pub fn select_takedown(&self) -> &Sound {
        let index = rand::thread_rng().gen_range(0..self.takedowns.len());
        &self.takedowns[index]
    }

    pub fn select_fail(&self) -> &Sound {
        let index = rand::thread_rng().gen_range(0..self.fails.len());
        &self.fails[index]
    }
}
