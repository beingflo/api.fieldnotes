mod authentication;
mod error;
mod notes;
mod schedule;
mod shares;
mod users;
mod util;

use axum::{
    routing::{delete, get, post, put},
    AddExtensionLayer, Router, Server,
};
use dotenv::dotenv;
use hyper::{header::CONTENT_TYPE, Method};
use log::{info, LevelFilter};
use schedule::{notes_deletion_schedule, tokens_deletion_schedule};
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use sqlx::postgres::PgPoolOptions;
use std::{fs::File, net::SocketAddr};
use tower_http::cors::{CorsLayer, Origin};

use crate::{
    notes::{
        delete_note_handler, get_note_handler, list_notes_handler, save_note_handler,
        undelete_note_handler, update_note_handler,
    },
    shares::{
        access_share_handler, create_share_handler, delete_share_handler,
        list_publications_handler, list_shares_handler,
    },
    users::{
        change_password_handler, delete_user_handler, invalidate_sessions, login_handler,
        logout_handler, signup_handler, store_salt_handler, user_info_handler,
    },
};

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
        .max_connections(100)
        .connect(&dotenv::var("DATABASE_URL").expect("DATABASE_URL env variable missing"))
        .await
        .expect("DB connection failed");

    let db = pool.clone();

    let write_origin = dotenv::var("WRITE_APP")
        .expect("WRITE_APP env variable missing")
        .as_str()
        .parse()
        .expect("WRITE_APP env variable malformed");
    let read_origin = dotenv::var("READ_APP")
        .expect("READ_APP env variable missing")
        .as_str()
        .parse()
        .expect("READ_APP env variable malformed");
    let read_www_origin = dotenv::var("READ_APP_WWW")
        .expect("READ_APP_WWW env variable missing")
        .as_str()
        .parse()
        .expect("READ_APP_WWW env variable malformed");

    let origins = Origin::list(vec![write_origin, read_origin, read_www_origin]);

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
        .layer(AddExtensionLayer::new(db))
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_credentials(true)
                .allow_headers(vec![CONTENT_TYPE]),
        );

    let listen: SocketAddr = dotenv::var("LISTEN")
        .expect("LISTEN env variable missing")
        .parse()
        .expect("Listen address invalid");

    let server = Server::bind(&listen).serve(app.into_make_service());

    let (_, _, _) = tokio::join!(
        server,
        notes_deletion_schedule(pool.clone()),
        tokens_deletion_schedule(pool.clone())
    );
}
