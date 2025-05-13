pub mod application_role_connection;
pub mod oauth;

const PORT: &str = macros::env!("DISCORD_VERIFY_PORT", "9000");
