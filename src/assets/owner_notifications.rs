use crate::assets::prepare_md;

macro_rules! msg {
    ($name:literal) => {
        prepare_md!(include_str!(concat!(
            "../../assets/owner_notifications/",
            $name,
            ".md"
        )))
    };
}

pub const MSG01_V3_UPDATE: &str = msg!("01-v3-update");
