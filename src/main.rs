mod authentication;
mod error;
mod notes;
mod schedule;
mod shares;
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

use crate::{users::{signup_handler, login_handler, delete_user_handler, change_password_handler, logout_handler, user_info_handler, invalidate_sessions, store_salt_handler}, notes::{list_notes_handler, get_note_handler, save_note_handler, update_note_handler, delete_note_handler, undelete_note_handler}, shares::{create_share_handler, list_shares_handler, delete_share_handler, access_share_handler, list_publications_handler}};

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
        .route("/notes/:token", delete(delete_note_handler))
        .route("/notes/undelete/:token", get(undelete_note_handler))
        .route("/shares", post(create_share_handler))
        .route("/shares", get(list_shares_handler))
        .route("/shares/:token", delete(delete_share_handler))
        .route("/shares/:token", get(access_share_handler))
        .route("/publications/:username", get(list_publications_handler))
        .layer(AddExtensionLayer::new(db));

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
