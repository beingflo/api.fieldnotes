use bcrypt::{hash, verify};
use serde::Deserialize;
use std::sync::Arc;
use warp::http::{Response, StatusCode};

use crate::util::{get_current_time, get_secure_token};
use log::{info, warn};
use warp::hyper::header::SET_COOKIE;
use warp::hyper::Body;
use warp::reject::Reject;
use warp::{Rejection, Reply};

use sqlx::PgPool;

use crate::endpoint;

/// Cost of bcrypt hashing algorithm. Low due to compute power on the target platform.
const BCRYPT_COST: u32 = 4;

/// Time in seconds for a session token to expire: 2 Months.
const TOKEN_EXPIRATION: u64 = 60 * 60 * 24 * 60;

/// This request form is expected for signup and login calls.
#[derive(Deserialize)]
pub struct UserCredentials {
    name: String,
    password: String,
}

/// Check if logged in and return available endpoints.
pub async fn me(token: Option<String>, pool: PgPool) -> Result<impl warp::Reply, warp::Rejection> {
    info!("Calling 'links' endpoint");

    let user_endpoints = endpoint::get_user_endpoints();
    let init_endpoints = endpoint::get_init_endpoints();

    // No username or token
    if token.is_none() {
        return Ok(warp::reply::json(&init_endpoints));
    }

    // Invalid username or token
    // TODO: Check in db if token valid
    if token.unwrap().is_empty() {
        return Ok(warp::reply::json(&init_endpoints));
    }

    // Valid username and token
    Ok(warp::reply::json(&user_endpoints))
}

/// Custom type to be used for 401 response.
#[derive(Debug)]
struct Unauthorized;

impl Reject for Unauthorized {}

/// Turn rejections into appropriate status codes
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(unauthorized) = err.find::<Unauthorized>() {
        info!("Recovering {:?} and returning UNAUTHORIZED", unauthorized);
        return Ok(StatusCode::UNAUTHORIZED);
    }

    if let Some(invalid_header) = err.find::<warp::reject::InvalidHeader>() {
        info!("Recovering {:?} and returning UNAUTHORIZED", invalid_header);
        return Ok(StatusCode::UNAUTHORIZED);
    }

    Err(err)
}
