use chrono::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use warp::http::{Response, StatusCode};
use warp::hyper::header::SET_COOKIE;
use warp::hyper::Body;

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

/// Get properly formatted cookie headers from name and token.
pub fn get_cookie_headers(token: &str, expiration: Duration) -> Response<Body> {
    let response = Response::builder().status(StatusCode::OK).header(
        SET_COOKIE,
        format!(
            "token={};HttpOnly;Max-Age={}",
            token,
            expiration.num_seconds()
        ),
    );

    response.body(Body::empty()).unwrap()
}

pub fn trucate_auth_token(token: &str) -> String {
    let length = token.len();
    let beginning = &token[..6];
    let end = &token[(length - 6)..];

    return format!("{}..{}", beginning, end);
}
