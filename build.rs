use std::{env, fs};
use std::path::PathBuf;

const LAVALINK_URL: &str = "https://github.com/freyacodes/Lavalink/releases/download/3.6.2/Lavalink.jar";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lavalink.application.yml");
    println!("cargo:rerun-if-env-changed=JAVA_HOME");

    // check if java is installed
    let java_home = java_locator::locate_java_home().expect("to find JAVA_HOME env variable");
    let mut java_exe: PathBuf = java_home.into();
    java_exe.push("bin");
    java_exe.push("java");
    #[cfg(windows)]
    java_exe.set_extension("exe");
    if !java_exe.exists() {
        panic!("expected java executable to be at {}", java_exe.to_string_lossy());
    }

    // make sure LavaLink is available
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut lavalink_local = PathBuf::from(manifest_dir);
    lavalink_local.push("Lavalink.jar");
    if !lavalink_local.exists() {
        let res = reqwest::blocking::get(LAVALINK_URL).expect("to GET LavaLink");
        let body = res.bytes().expect("to read bytes from GET response");
        fs::write(&lavalink_local, body).expect("to write LavaLink to disk");
    }
}
