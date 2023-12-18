use static_toml::static_toml;

pub mod help_texts;
pub mod owner_notifications;
pub mod sounds;

static_toml! {
    pub static REPLIES = include_toml!("assets/replies.toml");
    pub static QUOTES = include_toml!("assets/quotes.toml");
    pub static COLORS = include_toml!("assets/colors.toml");
}

macro_rules! prepare_md {
    ($in:expr) => {{
        use const_format::str_replace;

        use crate::util::trim::trim;

        str_replace!(
            str_replace!(
                str_replace!(
                    str_replace!(str_replace!(trim!($in), "\r\n", "\n"), "\n\n\n", "<br><br>"),
                    "\n\n",
                    "<br>"
                ),
                "\n",
                " "
            ),
            "<br>",
            "\n"
        )
    }};
}

pub(crate) use prepare_md;
