use chrono::Duration;
use rand::distributions::Alphanumeric;
use rand::Rng;
use warp::http::{Response, StatusCode};
use warp::hyper::header::SET_COOKIE;
use warp::hyper::Body;

/// Number of alphanumeric chars in auth tokens
const AUTH_TOKEN_LENGTH: usize = 64;

/// Number of alphanumeric chars in note tokens
const NOTE_TOKEN_LENGTH: usize = 16;

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
