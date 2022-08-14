mod definitions;
mod functions;
mod jobs;
mod logs;
mod roblox;
mod routes;

use actix_web::middleware::{self, Logger};
use actix_web::{get, web, App, HttpServer};
use anyhow;
use definitions::global_state::{AppState, Leaderboard};
use env_logger::Env;
use firebase_realtime_database::Database;
use functions::lb::write_users;
use parking_lot::RwLock;
use routes::configure_routes;

use std::{fs::File, io::Read};

#[get("/")]
async fn index() -> String {
    format!("wAVE mainframe backend extension!")
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let mut cookie_file = File::open("wij-games-cookie.txt")?;
    let mut cookie = String::new();
    cookie_file.read_to_string(&mut cookie).unwrap();

    let job_db = Database::from_path("wave-mainframe-default-rtdb", "firebase-key.json")?;
    // write_users(&job_db).await?;
    jobs::start_jobs(job_db);

    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .parse_env(Env::default().default_filter_or("info"))
        .init();

    let user = roblox::create_user(cookie, true).await;
    HttpServer::new(move || {
        let main_db =
            Database::from_path("wave-mainframe-default-rtdb", "firebase-key.json").unwrap();
        let lb = Leaderboard::new();

        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::NormalizePath::trim())
            .app_data(web::Data::new(AppState {
                database: RwLock::new(main_db),
                roblox_user: RwLock::new(user.clone()),
                leaderboard: RwLock::new(lb),
            }))
            .service(index)
            .configure(configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
