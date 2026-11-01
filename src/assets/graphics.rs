pub mod user_rank {
    pub const TEMPLATE: &[u8] = include_bytes!("../../assets/graphics/user_rank/template.png");

    #[rustfmt::skip]
    pub mod badges {
        pub const BLENDER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/blender.png");
        pub const IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/immunity.png");
        pub const NEVER_INITIATED: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/never_initiated.png");
        pub const NEVER_TAKEN_DOWN: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/never_taken_down.png");
        pub const RANDOM_IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/random_immunity.png");
        pub const TOP_COUNTERER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_counterer.png");
        pub const TOP_STRIKER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_striker.png");
        pub const TOP_VICTIM: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_victim.png");

        #[rustfmt::skip]
        pub mod emoji {
            pub const IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/immunity.png");
            pub const NEVER_INITIATED: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/never_initiated.png");
            pub const NEVER_TAKEN_DOWN: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/never_taken_down.png");
            pub const RANDOM_IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/random_immunity.png");
            pub const TOP_COUNTERER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/top_counterer.png");
            pub const TOP_STRIKER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/top_striker.png");
            pub const TOP_VICTIM: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/emoji/top_victim.png");
        }
    }
}
