#![allow(dead_code)]

mod ranks;
mod roblox;
mod users;

use actix_web::{get, web, App, HttpServer};
use firebase_realtime_database::*;
use std::io::Result;
use users::configure_users;

// This struct represents state
struct AppState {
    database: Database,
}

#[get("/")]
async fn index(_data: web::Data<AppState>) -> String {
    format!("Hello!") // <- response with app_name
}

#[actix_web::main]
async fn main() -> Result<()> {
    let token = get_oauth_token("firebase-key.json").await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                database: create_database("wave-mainframe-default-rtdb", token.as_str()),
            }))
            .service(index)
            .configure(configure_users)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
