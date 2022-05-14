mod api_down_queue;
mod key_generation;
mod promotion;
mod ranks;
mod roblox;
mod users;
mod verify;
mod verify_key_cleanup;

use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpServer};
use api_down_queue::start_queue_jobs;
use env_logger::Env;
use firebase_realtime_database::{create_database, get_oauth_token, Database};
use key_generation::init_keys;
use roblox::RobloxAccount;
use std::{
    fs::File,
    io::{Read, Result},
    sync::Mutex,
};
use users::configure_users;
use verify::configure_verify;
use verify_key_cleanup::start_verify_jobs;

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

    start_verify_jobs(job_database.clone());
    start_queue_jobs(job_database, job_user);
    init_keys();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .app_data(web::Data::new(Mutex::new(AppState {
                database: create_database("wave-mainframe-default-rtdb", token.as_str()),
                roblox_user: user.clone(),
            })))
            .service(index)
            .configure(configure_users)
            .configure(configure_verify)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
