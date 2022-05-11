mod api_down_queue;
mod promotion;
mod ranks;
mod roblox;
mod users;

use actix_web::{get, web, App, HttpServer};
use api_down_queue::start_jobs;
use firebase_realtime_database::{create_database, get_oauth_token, Database};
use roblox::RobloxAccount;
use std::{
    fs::File,
    io::{Read, Result},
    sync::Mutex,
};
use users::configure_users;

// This struct represents state
struct AppState {
    database: Database,
    roblox_user: RobloxAccount,
}

#[get("/")]
async fn index() -> String {
    format!("wAVE mainframe backend extension!")
}

#[actix_web::main]
async fn main() -> Result<()> {
    let mut cookie_file = File::open("wij-games-cookie.txt")?;
    let mut cookie = String::new();
    cookie_file.read_to_string(&mut cookie).unwrap();

    let user = roblox::create_user(cookie, true).await;
    let job_user = user.clone();

    let token = get_oauth_token("firebase-key.json").await.unwrap();
    let job_database = create_database("wave-mainframe-default-rtdb", token.as_str());
    start_jobs(job_database, job_user);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Mutex::new(AppState {
                database: create_database("wave-mainframe-default-rtdb", token.as_str()),
                roblox_user: user.clone(),
            })))
            .service(index)
            .configure(configure_users)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
