use log::info;
use sqlx::PgPool;

use crate::endpoint;

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
