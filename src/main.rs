mod authentication;
mod error;
#[macro_use]
mod helper;
mod notes;
mod schedule;
mod shares;
mod users;
mod util;

use dotenv::dotenv;
use log::{LevelFilter, info};
use schedule::{notes_deletion_schedule, tokens_deletion_schedule};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use sqlx::postgres::PgPoolOptions;
use std::{fs::File, net::SocketAddr};
use warp::Filter;

use crate::error::handle_rejection;

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_time_format_str("%F %T")
                .add_filter_allow_str("textli")
                .set_time_to_local(true)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_time_format_str("%F %T")
                .set_time_to_local(true)
                .build(),
            File::create("textli.log").unwrap(),
        ),
    ])
    .unwrap();

    info!("Server started");

    dotenv().ok();

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&dotenv::var("DATABASE_URL").expect("DATABASE_URL env variable missing"))
        .await
        .expect("DB connection failed");

    let db_clone = pool.clone();
    let with_db = warp::any().map(move || db_clone.clone());

    let log = warp::log::custom(|info| {
        info!("{} {} {}", info.method(), info.path(), info.status(),);
    });

    let with_token = warp::filters::cookie::cookie("token");

    let with_user = with_token
        .and(with_db.clone())
        .and_then(authentication::is_authorized_with_user);

    let is_authorized_with_user = with_token
        .and(with_db.clone())
        .and_then(authentication::is_authorized_with_user);

    let is_funded = with_user
        .clone()
        .and(with_db.clone())
        .and_then(users::is_funded)
        .untuple_one();

    let signup = warp::post()
        .and(warp::path("user"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db.clone())
        .then(users::signup_handler)
        .and_then(error::handle_errors);

    let change_password = warp::put()
        .and(warp::path("user"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(users::change_password_handler)
        .and_then(error::handle_errors);

    let logout = warp::delete()
        .and(warp::path("session"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_token)
        .and(with_db.clone())
        .then(users::logout_handler)
        .and_then(error::handle_errors);

    let delete_user = warp::delete()
        .and(warp::path("user"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(users::delete_user_handler)
        .and_then(error::handle_errors);

    let login = warp::post()
        .and(warp::path("session"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_db.clone())
        .then(users::login_handler)
        .and_then(error::handle_errors);

    let user_info = warp::get()
        .and(warp::path!("user" / "info"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(users::user_info_handler)
        .and_then(error::handle_errors);

    let invalidate_all_sessions = warp::delete()
        .and(warp::path("allsessions"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(users::invalidate_sessions)
        .and_then(error::handle_errors);

    let store_salt = warp::put()
        .and(warp::path!("user" / "salt"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .then(users::store_salt_handler)
        .and_then(error::handle_errors);

    let user_api = login
        .or(signup)
        .or(logout)
        .or(change_password)
        .or(store_salt)
        .or(delete_user)
        .or(user_info)
        .or(invalidate_all_sessions);

    let list_notes = warp::get()
        .and(warp::path("notes"))
        .and(warp::path::end())
        .and(warp::query::query())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(notes::list_notes_handler)
        .and_then(error::handle_errors);

    let get_note = warp::get()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(notes::get_note_handler)
        .and_then(error::handle_errors);

    let save_note = warp::post()
        .and(warp::path("notes"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(is_funded.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .then(notes::save_note_handler)
        .and_then(error::handle_errors);

    let update_note = warp::put()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(is_funded.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .then(notes::update_note_handler)
        .and_then(error::handle_errors);

    let delete_note = warp::delete()
        .and(warp::path!("notes" / String))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(notes::delete_note_handler)
        .and_then(error::handle_errors);

    let undelete_note = warp::get()
        .and(warp::path!("notes" / "undelete" / String))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(notes::undelete_note_handler)
        .and_then(error::handle_errors);

    let note_api = get_note
        .or(list_notes)
        .or(save_note)
        .or(update_note)
        .or(delete_note)
        .or(undelete_note);

    let create_share = warp::post()
        .and(warp::path("shares"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(is_funded.clone())
        .and(warp::body::json())
        .and(with_db.clone())
        .then(shares::create_share_handler)
        .and_then(error::handle_errors);

    let list_shares = warp::get()
        .and(warp::path("shares"))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(shares::list_shares_handler)
        .and_then(error::handle_errors);

    let delete_share = warp::delete()
        .and(warp::path!("shares" / String))
        .and(warp::path::end())
        .and(is_authorized_with_user.clone())
        .and(with_db.clone())
        .then(shares::delete_share_handler)
        .and_then(error::handle_errors);

    // Non-authorized access allowed here
    let access_share = warp::get()
        .and(warp::path!("shares" / String))
        .and(warp::path::end())
        .and(with_db.clone())
        .then(shares::access_share_handler)
        .and_then(error::handle_errors);

    let share_api = create_share
        .or(delete_share)
        .or(list_shares)
        .or(access_share);

    // Non-authorized access allowed here
    let list_publications = warp::get()
        .and(warp::path!("publications" / String))
        .and(warp::path::end())
        .and(with_db.clone())
        .then(shares::list_publications_handler)
        .and_then(error::handle_errors);

    let cors = warp::cors()
        .allow_origins([
            dotenv::var("WRITE_APP")
                .expect("WRITE_APP env variable missing")
                .as_str(),
            dotenv::var("READ_APP")
                .expect("READ_APP env variable missing")
                .as_str(),
        ])
        .allow_headers(vec!["content-type"])
        .allow_credentials(true)
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .build();

    let listen: SocketAddr = dotenv::var("LISTEN")
        .expect("LISTEN env variable missing")
        .parse()
        .expect("Listen address invalid");

    tokio::join!(
        warp::serve(
            combine!(user_api, note_api, share_api, list_publications)
                .recover(handle_rejection)
                .with(cors)
                .with(log),
        )
        .run(listen),
        notes_deletion_schedule(pool.clone()),
        tokens_deletion_schedule(pool.clone())
    );
}
