mod authentication;
mod error;
#[macro_use]
mod helper;
mod note;
mod schedule;
mod share;
mod user;
mod util;

use dotenv::dotenv;
use error::handle_rejection;
use log::info;
use schedule::{balance_decrease_schedule, notes_deletion_schedule, tokens_deletion_schedule};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use warp::Filter;

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

    let db_clone = pool.clone();
    let with_db = warp::any().map(move || db_clone.clone());

    let with_token = warp::filters::cookie::cookie("token");

    let with_user = with_token
        .and(with_db.clone())
        .and_then(authentication::get_user_id);

    let is_authorized = with_token
        .and(with_db.clone())
        .and_then(authentication::is_authorized)
        .untuple_one();

    let is_funded = with_user
        .clone()
        .and(with_db.clone())
        .and_then(user::is_funded)
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

    let user_info = warp::get()
        .and(warp::path!("user" / "info"))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(user::user_info_handler);

    let list_notes = warp::get()
        .and(warp::path("notes"))
        .and(warp::path::end())
        .and(warp::query::query())
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
        .and(is_funded.clone())
        .and(with_user.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(note::save_note_handler);

    let update_note = warp::put()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(is_funded.clone())
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

    let create_share = warp::post()
        .and(warp::path("share"))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(is_funded.clone())
        .and(with_user.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .and_then(share::create_share_handler);

    let list_shares = warp::get()
        .and(warp::path("share"))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(is_funded.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(share::list_shares_handler);

    let delete_share = warp::delete()
        .and(warp::path!("share" / String))
        .and(warp::path::end())
        .and(is_authorized.clone())
        .and(with_user.clone())
        .and(with_db.clone())
        .and_then(share::delete_share_handler);

    // Non-authorized access allowed here
    let access_share = warp::get()
        .and(warp::path!("share" / String))
        .and(warp::path::end())
        .and(with_db.clone())
        .and_then(share::access_share_handler);

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

    tokio::join!(
        warp::serve(
            combine!(
                signup,
                login,
                logout,
                delete_user,
                user_info,
                change_password,
                list_notes,
                get_note,
                save_note,
                update_note,
                delete_note,
                undelete_note,
                create_share,
                delete_share,
                access_share,
                list_shares
            )
            .with(cors)
            .recover(handle_rejection),
        )
        .run(listen),
        balance_decrease_schedule(pool.clone()),
        notes_deletion_schedule(pool.clone()),
        tokens_deletion_schedule(pool.clone())
    );
}
