use firebase_realtime_database::Database;
use std::{thread, time::Duration};
use tokio::task;

mod verify_key_cleanup;

static THREAD_DELAY: u64 = 30000;

pub fn start_jobs(database: Database) {
    task::spawn(async move {
        let db = database;
        loop {
            verify_key_cleanup::key_cleanup(&db).await;

            thread::sleep(Duration::from_millis(THREAD_DELAY));
        }
    });
}
