mod definitions;
mod functions;
mod jobs;
mod logs;
mod roblox;
mod routes;

use actix_web::middleware::{self, Logger};
use actix_web::{get, web, App, HttpServer};
use env_logger::Env;
use firebase_realtime_database::{create_database, get_oauth_token, Database};
use roblox::RobloxAccount;
use routes::configure_routes;
use std::{
    fs::File,
    io::{Read, Result},
    sync::RwLock,
};

// This struct represents state
struct AppState {
    database: Database,
    roblox_user: RwLock<RobloxAccount>,
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

    let token = get_oauth_token("firebase-key.json").await.unwrap();
    let job_db = create_database("wave-mainframe-default-rtdb", token.as_str());

    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .parse_env(Env::default().default_filter_or("info"))
        .init();

    jobs::start_jobs(job_db);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::NormalizePath::trim())
            .app_data(web::Data::new(AppState {
                database: create_database("wave-mainframe-default-rtdb", token.as_str()),
                roblox_user: RwLock::new(user.clone()),
            }))
            .service(index)
            .configure(configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
