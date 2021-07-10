mod authentication;
mod endpoint;
mod error;
mod user;
mod util;

use log::info;
use sqlx::postgres::PgPoolOptions;
use warp::Filter;

use std::net::SocketAddr;

use dotenv::dotenv;

use error::handle_rejection;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Textli started");

    dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&dotenv::var("DATABASE_URL").expect("DATABASE_URL env variable missing"))
        .await
        .expect("DB connection failed");

    let with_db = warp::any().map(move || pool.clone());

    let is_authorized = warp::filters::cookie::cookie("token")
        .and(with_db.clone())
        .and_then(authentication::is_authorized);

    let signup = warp::post()
        .and(warp::path("user"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(user::signup);

    let test = warp::get()
        .and(warp::path("test"))
        .and(is_authorized.clone())
        .and(with_db.clone())
        .and_then(user::test);

    let login = warp::post()
        .and(warp::path("session"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(user::login);

    let cors = warp::cors()
        .allow_origin(
            dotenv::var("ALLOW_ORIGIN")
                .expect("ALLOW_ORIGIN env variable missing")
                .as_str(),
        )
        .allow_headers(vec!["content-type"])
        .allow_credentials(true)
        .allow_methods(vec!["GET", "POST", "PATCH", "DELETE"]);

    let listen: SocketAddr = dotenv::var("LISTEN")
        .expect("LISTEN env variable missing")
        .parse()
        .expect("Listen address invalid");

    warp::serve(
        signup
            .or(test)
            .or(login)
            .with(cors)
            .recover(handle_rejection),
    )
    .run(listen)
    .await;
}
