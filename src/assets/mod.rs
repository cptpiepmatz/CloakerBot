use static_toml::static_toml;

#[derive(Debug, Clone, Copy)]
pub struct MarkdownSection {
    pub title: &'static str,
    pub heading_level: u8,
    pub content: &'static str
}

pub mod graphics;
pub mod help_texts;
pub mod oauth;
pub mod owner_notifications;
pub mod sounds;

macros::template_toml!("../../assets/replies.toml");

static_toml! {
    pub static QUOTES = include_toml!("assets/quotes.toml");
    pub static COLORS = include_toml!("assets/colors.toml");
}
