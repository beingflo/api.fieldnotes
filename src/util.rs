use axum::http::{self, HeaderValue};
use chrono::Duration;
use hyper::HeaderMap;
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::error::AppError;

/// Number of alphanumeric chars in auth tokens
const AUTH_TOKEN_LENGTH: usize = 128;

/// Number of alphanumeric chars in share tokens
const SHARE_TOKEN_LENGTH: usize = 32;

/// Number of alphanumeric chars in note tokens
const NOTE_TOKEN_LENGTH: usize = 32;

/// Get a secure token for session tokens
pub fn get_auth_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

/// Get a secure token for share tokens
pub fn get_share_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(SHARE_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

/// Get a secure token for note ids
pub fn get_note_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(NOTE_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

pub fn truncate_auth_token(token: &str) -> String {
    let length = token.len();
    let beginning = &token[..6];
    let end = &token[(length - 6)..];

    return format!("{}..{}", beginning, end);
}

pub fn get_token_from_header(headers: &HeaderMap) -> Result<String, AppError> {
    if let Some(cookie) = headers
        .get(http::header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
    {
        let mut split = cookie.split("=");
        split.next();
        match split.next() {
            Some(str) => return Ok(str.into()),
            None => return Err(AppError::Unauthorized),
        }
    } else {
        return Err(AppError::Unauthorized);
    };
}

pub fn get_header_with_token(token: &str, duration: Duration) -> HeaderMap {
    let cookie = format!(
        "token={};HttpOnly;Secure;SameSite=Strict;Max-Age={}",
        token,
        duration.num_seconds()
    );

    let cookie_header = HeaderValue::from_str(&cookie).expect("Cookie value invalid");

    let mut headers = HeaderMap::new();
    headers.insert(http::header::SET_COOKIE, cookie_header);

    headers
}
