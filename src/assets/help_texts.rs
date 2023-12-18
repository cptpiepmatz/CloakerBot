use crate::assets::prepare_md;

macro_rules! help_text {
    ($name:literal) => {
        prepare_md!(include_str!(concat!("../../assets/help/", $name, ".md")))
    };
}

pub const CLOAKER: &str = help_text!("cloaker");
pub const CONFIG_DANGER_ZONES: &str = help_text!("config_danger_zones");
pub const CONFIG_IMMUNITY: &str = help_text!("config_immunity");
pub const CONFIG_LOG_CHANNEL: &str = help_text!("config_log_channel");
pub const CONFIG_RANDOM: &str = help_text!("config_random");
pub const COUNTER: &str = help_text!("counter");

// TODO: write a help text for ranks command
pub const RANKS: &str = "";
