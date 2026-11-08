use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

static OUT_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env::var("OUT_DIR").expect("set by cargo")));
static_toml::static_toml! {
    static MANIFEST = include_toml!("Cargo.toml");
}

fn main() {
    sounds::regain_audio_files();
    git::detect_branch();
}

mod sounds {
    use std::path::Path;
    use std::{fs, process};

    use hound::{SampleFormat, WavReader, WavWriter};

    use super::*;

    const AUDIO_FILES: &[&str] = &[
        "fails/pain1.wav",
        "fails/pain10.wav",
        "fails/pain11.wav",
        "fails/pain12.wav",
        "fails/pain13.wav",
        "fails/pain14.wav",
        "fails/pain2.wav",
        "fails/pain3.wav",
        "fails/pain4.wav",
        "fails/pain5.wav",
        "fails/pain6.wav",
        "fails/pain7.wav",
        "fails/pain8.wav",
        "fails/pain9.wav",
        "takedowns/beat the shit.wav",
        "takedowns/better late than never.wav",
        "takedowns/check your corners.wav",
        "takedowns/come back for more.wav",
        "takedowns/cry for mom.wav",
        "takedowns/didn't forget about me.wav",
        "takedowns/difficulty tweak.wav",
        "takedowns/drum solo.wav",
        "takedowns/go to the forums.wav",
        "takedowns/gotta stop meeting like this.wav",
        "takedowns/guess you're not as good.wav",
        "takedowns/i bet you let yourself get beat up.wav",
        "takedowns/i expected better.wav",
        "takedowns/i'm gonna beat the.wav",
        "takedowns/i'm late.wav",
        "takedowns/i've got your dlc.wav",
        "takedowns/i've got your safehouse.wav",
        "takedowns/missed me.wav",
        "takedowns/nice workout.wav",
        "takedowns/not as tough as they told me.wav",
        "takedowns/not so tough now.wav",
        "takedowns/point of no return.wav",
        "takedowns/put up a better fight.wav",
        "takedowns/putting up a fight.wav",
        "takedowns/resisting arrest.wav",
        "takedowns/speak up.wav",
        "takedowns/stop hitting yourself.wav",
        "takedowns/that one's gonna hurt.wav",
        "takedowns/the safeword.wav",
        "takedowns/this might sting a little.wav",
        "takedowns/try a little harder.wav",
        "takedowns/turn you inside out.wav",
        "takedowns/we gotta do this more often.wav",
        "takedowns/wear that shit in your pants proudly.wav",
        "takedowns/who's the clown now.wav",
        "takedowns/wishes do come true.wav",
        "takedowns/work smarter.wav",
        "takedowns/you asked for it.wav",
        "takedowns/you got what was coming.wav",
        "takedowns/you wanted me back.wav",
        "WULULULULU.wav"
    ];

    const GAIN: f32 = 0.15;

    fn regain_audio_file(input: impl AsRef<Path>, output: impl AsRef<Path>) {
        let reader = WavReader::open(input).unwrap();
        let spec = reader.spec();
        let mut writer = WavWriter::create(output, spec).unwrap();

        if spec.sample_format != SampleFormat::Int || spec.bits_per_sample != 16 {
            eprintln!("Only 16-bit PCM is handled.");
            process::exit(1);
        }

        for sample in reader.into_samples::<i16>() {
            let sample = sample.unwrap();
            let scaled = (sample as f32 * GAIN).clamp(i16::MIN as f32, i16::MAX as f32);
            writer.write_sample(scaled as i16).unwrap();
        }

        writer.finalize().unwrap();
    }

    pub fn regain_audio_files() {
        for audio_file in AUDIO_FILES {
            println!("cargo::rerun-if-changed=assets/sounds/{audio_file}");
            let path = OUT_DIR.join("sounds").join(audio_file);
            let dirname = path.parent().expect("files have a dir");
            fs::create_dir_all(dirname).unwrap();
            regain_audio_file("assets/sounds/".to_string() + audio_file, path);
        }
    }
}

mod git {
    use std::fs;
    use std::process::Command;

    use quote::quote;

    use super::*;

    pub fn detect_branch() {
        println!("cargo::rerun-if-changed=.git/HEAD");
        let branch = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .map(|output| {
                let output = String::from_utf8(output.stdout)
                    .expect("git should return valid utf8")
                    .trim()
                    .to_owned();
                match output.len() {
                    0 => MANIFEST.package.metadata.git.default_branch.to_owned(),
                    _ => output
                }
            })
            .unwrap_or_else(|_| MANIFEST.package.metadata.git.default_branch.to_owned());

        let code = quote! {
            mod git {
                pub const BRANCH: &str = #branch;
            }
        };

        fs::write(OUT_DIR.join("git.rs"), code.to_string()).unwrap();
    }
}
