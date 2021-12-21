mod authentication;
mod error;
mod notes;
mod schedule;
//mod shares;
mod users;
mod util;

use dotenv::dotenv;
use log::{info, LevelFilter};
use schedule::{notes_deletion_schedule, tokens_deletion_schedule};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use sqlx::postgres::PgPoolOptions;
use std::{fs::File, net::SocketAddr};
use axum::{Server, Router, routing::{post, delete, put, get}, AddExtensionLayer};

use crate::{users::{signup_handler, login_handler, delete_user_handler, change_password_handler, logout_handler, user_info_handler, invalidate_sessions, store_salt_handler}, notes::{list_notes_handler, get_note_handler, save_note_handler, update_note_handler}};

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new()
                .set_time_format_str("%F %T")
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
            File::create("fieldnotes.log").unwrap(),
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

    let db = pool.clone();

    let app = Router::new()
        .route("/user", post(signup_handler))
        .route("/session", post(login_handler))
        .route("/user", delete(delete_user_handler))
        .route("/user", put(change_password_handler))
        .route("/session", delete(logout_handler))
        .route("/user/info", get(user_info_handler))
        .route("/user/salt", put(store_salt_handler))
        .route("/allsessions", delete(invalidate_sessions))
        .route("/notes", get(list_notes_handler))
        .route("/notes/:token", get(get_note_handler))
        .route("/notes", post(save_note_handler))
        .route("/notes/:token", put(update_note_handler))
        .layer(AddExtensionLayer::new(db));

    //let update_note = warp::put()
    //    .and(warp::path!("notes" / String))
    //    .and(warp::path::end())
    //    .and(is_authorized_with_user.clone())
    //    .and(is_funded.clone())
    //    .and(warp::body::json())
    //    .and(with_db.clone())
    //    .then(notes::update_note_handler)
    //    .and_then(error::handle_errors);

    //let delete_note = warp::delete()
    //    .and(warp::path!("notes" / String))
    //    .and(warp::path::end())
    //    .and(is_authorized_with_user.clone())
    //    .and(with_db.clone())
    //    .then(notes::delete_note_handler)
    //    .and_then(error::handle_errors);

    //let undelete_note = warp::get()
    //    .and(warp::path!("notes" / "undelete" / String))
    //    .and(warp::path::end())
    //    .and(is_authorized_with_user.clone())
    //    .and(with_db.clone())
    //    .then(notes::undelete_note_handler)
    //    .and_then(error::handle_errors);

    //let note_api = get_note
    //    .or(list_notes)
    //    .or(save_note)
    //    .or(update_note)
    //    .or(delete_note)
    //    .or(undelete_note);

    //let create_share = warp::post()
    //    .and(warp::path("shares"))
    //    .and(warp::path::end())
    //    .and(is_authorized_with_user.clone())
    //    .and(is_funded.clone())
    //    .and(warp::body::json())
    //    .and(with_db.clone())
    //    .then(shares::create_share_handler)
    //    .and_then(error::handle_errors);

    //let list_shares = warp::get()
    //    .and(warp::path("shares"))
    //    .and(warp::path::end())
    //    .and(is_authorized_with_user.clone())
    //    .and(with_db.clone())
    //    .then(shares::list_shares_handler)
    //    .and_then(error::handle_errors);

    //let delete_share = warp::delete()
    //    .and(warp::path!("shares" / String))
    //    .and(warp::path::end())
    //    .and(is_authorized_with_user.clone())
    //    .and(with_db.clone())
    //    .then(shares::delete_share_handler)
    //    .and_then(error::handle_errors);

    //// Non-authorized access allowed here
    //let access_share = warp::get()
    //    .and(warp::path!("shares" / String))
    //    .and(warp::path::end())
    //    .and(with_db.clone())
    //    .then(shares::access_share_handler)
    //    .and_then(error::handle_errors);

    //let share_api = create_share
    //    .or(delete_share)
    //    .or(list_shares)
    //    .or(access_share);

    //// Non-authorized access allowed here
    //let list_publications = warp::get()
    //    .and(warp::path!("publications" / String))
    //    .and(warp::path::end())
    //    .and(with_db.clone())
    //    .then(shares::list_publications_handler)
    //    .and_then(error::handle_errors);

    //let cors = warp::cors()
    //    .allow_origins([
    //        dotenv::var("WRITE_APP")
    //            .expect("WRITE_APP env variable missing")
    //            .as_str(),
    //        dotenv::var("READ_APP")
    //            .expect("READ_APP env variable missing")
    //            .as_str(),
    //        dotenv::var("READ_APP_WWW")
    //            .expect("READ_APP_WWW env variable missing")
    //            .as_str(),
    //    ])
    //    .allow_headers(vec!["content-type"])
    //    .allow_credentials(true)
    //    .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
    //    .build();

    let listen: SocketAddr = dotenv::var("LISTEN")
        .expect("LISTEN env variable missing")
        .parse()
        .expect("Listen address invalid");

    let server = Server::bind(&listen)
        .serve(app.into_make_service());

    let (_,_,_) = tokio::join!(
        server,
        notes_deletion_schedule(pool.clone()),
        tokens_deletion_schedule(pool.clone())
    );
}
