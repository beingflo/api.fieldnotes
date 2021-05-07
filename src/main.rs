mod authentication;
mod endpoint;
mod user;
mod util;

use log::info;
use sqlx::postgres::PgPoolOptions;
use warp::Filter;

use std::net::SocketAddr;

use dotenv::dotenv;

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

    let me = warp::get()
        .and(warp::path("me"))
        .and(warp::path::end())
        .and(warp::filters::cookie::optional("token"))
        .and(with_db.clone())
        .and_then(authentication::me);

    let signup = warp::post()
        .and(warp::path("user"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(user::signup);

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

    warp::serve(me.or(signup).with(cors)).run(listen).await;
}
