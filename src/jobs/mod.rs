use std::{thread, time::Duration};

use firebase_realtime_database::Database;

use crate::roblox::RobloxAccount;
use tokio::task;

pub mod api_down_queue;
// mod in_group_queue;
mod verify_key_cleanup;

static THREAD_DELAY: u64 = 30000;

pub fn start_jobs(database: Database, mut user: RobloxAccount) {
    task::spawn(async move {
        loop {
            verify_key_cleanup::key_cleanup(&database).await;
            api_down_queue::queue_handler(&database, &mut user).await;

            thread::sleep(Duration::from_millis(THREAD_DELAY));
        }
    });
}
