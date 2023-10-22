use static_toml::static_toml;

pub mod sounds;

static_toml! {
    pub static REPLIES = include_toml!("assets/replies.toml");
    pub static COLORS = include_toml!("assets/colors.toml");
}
