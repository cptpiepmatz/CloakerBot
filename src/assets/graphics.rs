pub mod user_rank {
    const TEMPLATE: &[u8] = include_bytes!("../../assets/graphics/user_rank/template.png");

    #[rustfmt::skip]
    pub mod badges {
        const BLENDER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/blender.png");
        const IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/immunity.png");
        const NEVER_INITIATED: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/never_initiated.png");
        const NEVER_TAKEN_DOWN: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/never_taken_down.png");
        const RANDOM_IMMUNITY: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/random_immunity.png");
        const TOP_COUNTERER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_counterer.png");
        const TOP_STRIKER: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_striker.png");
        const TOP_VICTIM: &[u8] = include_bytes!("../../assets/graphics/user_rank/badges/top_victim.png");
    }
}
