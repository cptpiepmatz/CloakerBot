pub mod user_rank {
    pub const BANNER: &[u8] = include_bytes!("../../assets/graphics/user_rank/banner.webp");

    #[rustfmt::skip]
    pub mod badges {
        pub const IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/immunity.webp");
        pub const NEVER_INITIATED: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/never_initiated.webp");
        pub const NEVER_TAKEN_DOWN: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/never_taken_down.webp");
        pub const RANDOM_IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/random_immunity.webp");
        pub const TOP_COUNTERER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_counterer.webp");
        pub const TOP_STRIKER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_striker.webp");
        pub const TOP_VICTIM: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_victim.webp");
    }
}
