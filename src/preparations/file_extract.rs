use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::{fs, iter};

use crate::assets::sounds;
use crate::embedded_files::{LAVALINK, LAVALINK_CONFIG};

pub fn extract_files() -> IoResult<()> {
    // lavalink
    fs::write(LAVALINK.0, LAVALINK.1)?;
    fs::write("application.yml", LAVALINK_CONFIG.1)?;

    // sound assets
    for file in sounds::fails::ALL
        .iter()
        .chain(sounds::takedowns::ALL)
        .chain(iter::once(&sounds::WULULULULU))
    {
        fs::create_dir_all(
            PathBuf::from(format!("sounds/{}", file.0))
                .parent()
                .expect("to find parent")
        )?;
        fs::write(format!("sounds/{}", file.0), file.1)?;
    }

    Ok(())
}
