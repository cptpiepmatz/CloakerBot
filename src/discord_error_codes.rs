//! Discord JSON error codes used by the bot.
//!
//! See <https://docs.discord.com/developers/topics/opcodes-and-status-codes#json-json-error-codes>.

/// Missing access.
pub const MISSING_ACCESS: u64 = 50001;

/// Cannot send messages to this user.
pub const CANNOT_SEND_MESSAGES_TO_THIS_USER: u64 = 50007;

/// Missing permissions.
pub const MISSING_PERMISSIONS: u64 = 50013;
