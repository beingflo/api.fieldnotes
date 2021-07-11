use rand::distributions::Alphanumeric;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::http::{Response, StatusCode};
use warp::hyper::header::SET_COOKIE;
use warp::hyper::Body;

/// Number of alphanumeric chars in auth tokens
const AUTH_TOKEN_LENGTH: usize = 64;

/// Number of alphanumeric chars in note tokens
const NOTE_TOKEN_LENGTH: usize = 16;

/// Get current time in seconds since Unix Epoch for timestamps.
pub fn get_current_time() -> i64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

/// Get a secure token for session tokens or share links.
pub fn get_auth_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(AUTH_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

/// Get a secure token for session tokens or share links.
pub fn get_note_token() -> String {
    rand::rngs::OsRng
        .sample_iter(&Alphanumeric)
        .take(NOTE_TOKEN_LENGTH)
        .map(char::from)
        .collect::<String>()
}

/// Get properly formatted cookie headers from name and token.
pub fn get_cookie_headers(token: &str, expiration: i64) -> Response<Body> {
    let response = Response::builder().status(StatusCode::OK).header(
        SET_COOKIE,
        format!("token={};HttpOnly;Max-Age={}", token, expiration),
    );

    response.body(Body::empty()).unwrap()
}
