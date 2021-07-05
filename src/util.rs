use rand::distributions::Alphanumeric;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

/// Number of alphanumeric chars in secure tokens
const TOKEN_LENGTH: usize = 32;

/// Get current time in seconds since Unix Epoch for timestamps.
pub fn get_current_time() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Get a secure token for session tokens or share links.
pub fn get_secure_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}
