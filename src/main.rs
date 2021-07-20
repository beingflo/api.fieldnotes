mod authentication;
mod endpoint;
mod error;
mod note;
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

    let with_token = warp::filters::cookie::cookie("token");

    let with_user = with_token
        .and(with_db.clone())
        .and_then(authentication::get_user_id);

    let is_authorized = with_token
        .and(with_db.clone())
        .and_then(authentication::is_authorized)
        .untuple_one();

    let signup = warp::post()
        .and(warp::path("user"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(user::signup);

    let change_password = warp::put()
        .and(warp::path("user"))
        .and(warp::body::json())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(user::change_password);

    let logout = warp::delete()
        .and(warp::path("session"))
        .and(is_authorized.clone())
        .and(with_token)
        .and(with_db.clone())
        .and_then(user::logout);

    let delete_user = warp::delete()
        .and(warp::path("user"))
        .and(warp::body::json())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(user::delete_user);

    let login = warp::post()
        .and(warp::path("session"))
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(user::login);

    let list_notes = warp::get()
        .and(warp::path("notes"))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(note::list_notes_handler);

    let get_note = warp::get()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(note::get_note_handler);

    let save_note = warp::post()
        .and(warp::path("notes"))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(note::save_note_handler);

    let update_note = warp::put()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(note::update_note_handler);

    let delete_note = warp::delete()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(note::delete_note_handler);

    let undelete_note = warp::post()
        .and(warp::path!("notes" / "undelete" / String))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(note::undelete_note_handler);

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
            .or(login)
            .or(logout)
            .or(delete_user)
            .or(change_password)
            .or(list_notes)
            .or(get_note)
            .or(save_note)
            .or(update_note)
            .or(delete_note)
            .or(undelete_note)
            .with(cors)
            .recover(handle_rejection),
    )
    .run(listen)
    .await;
}
