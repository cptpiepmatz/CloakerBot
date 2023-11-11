use const_format::{concatcp, formatcp, str_replace};
use crate::util::trim::trim;

macro_rules! prepare_md {
    ($in:expr) => {
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
    };
}

pub const CLOAKER: &str = prepare_md!(include_str!("../../assets/help/cloaker.md"));

pub const CONFIG_DANGER_ZONES: &str = prepare_md!(include_str!("../../assets/help/config_danger_zones.md"));

pub const CONFIG_IMMUNITY: &str = prepare_md!(include_str!("../../assets/help/config_immunity.md"));

pub const CONFIG_LOG_CHANNEL: &str = prepare_md!(include_str!("../../assets/help/config_log_channel.md"));

pub const CONFIG_TARGET_BOTS: &str = prepare_md!(include_str!("../../assets/help/config_target_bots.md"));

pub const CONFIG_RANDOM: &str = prepare_md!(include_str!("../../assets/help/config_random.md"));

pub const COUNTER: &str = prepare_md!(include_str!("../../assets/help/counter.md"));
