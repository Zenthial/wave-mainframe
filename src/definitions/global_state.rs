use crate::{
    functions::lb::{read_users, write_users},
    roblox::RobloxAccount,
};
use firebase_realtime_database::Database;
use log::info;
use parking_lot::RwLock;
use std::time::Instant;

use super::users::User;

const LB_REFRESH_TIME: u64 = 60 * 60 * 60;

#[derive(Debug, Clone)]
pub struct Leaderboard {
    last_update: Instant,
    sorted: Vec<User>,
}

impl Leaderboard {
    pub fn new() -> Self {
        let mut sorted = vec![];

        let result = read_users();
        if result.is_ok() {
            sorted = result.unwrap();
        }

        Leaderboard {
            last_update: Instant::now(),
            sorted,
        }
    }

    pub async fn update(&mut self, db: &Database) {
        let result = write_users(db).await;

        if result.is_err() {
            info!("{:?}", result.unwrap_err());
        }

        match read_users() {
            Ok(new_sorted) => self.sorted = new_sorted,
            Err(e) => {
                info!("{:?}", e);
            }
        }
    }

    pub fn needs_update(&self) -> bool {
        if self.last_update.elapsed().as_secs() >= LB_REFRESH_TIME {
            true
        } else {
            false
        }
    }

    pub fn get(&self) -> &Vec<User> {
        &self.sorted
    }
}

// This struct represents state
pub struct AppState {
    pub database: RwLock<Database>,
    pub roblox_user: RwLock<RobloxAccount>,
    pub leaderboard: RwLock<Leaderboard>,
}
