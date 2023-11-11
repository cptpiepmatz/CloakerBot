use static_toml::static_toml;

pub mod sounds;
pub mod help_texts;

static_toml! {
    pub static REPLIES = include_toml!("assets/replies.toml");
    pub static QUOTES = include_toml!("assets/quotes.toml");
    pub static COLORS = include_toml!("assets/colors.toml");
}
