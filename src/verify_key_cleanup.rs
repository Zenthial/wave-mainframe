use std::{
    collections::HashMap,
    thread,
    time::{Duration, SystemTime},
};

use firebase_realtime_database::Database;
use tokio::task;

use crate::verify::VerificationCodeBody;

static THREAD_DELAY: u64 = 60000;
static ONE_HOUR: u64 = 600000;

async fn key_cleanup(database: &Database) {
    let key_result = database.get("verification/keys/").await;

    if let Ok(response) = key_result {
        let user_map_option = response
            .json::<Option<HashMap<String, VerificationCodeBody>>>()
            .await
            .unwrap();
        if let Some(user_map) = user_map_option {
            for (key, user) in user_map {
                let current_time = SystemTime::now();
                if current_time.duration_since(user.creation_time).unwrap()
                    >= Duration::from_millis(ONE_HOUR)
                {
                    let delete_response = database
                        .delete(format!("verification/keys/{}", key).as_str())
                        .await;

                    if let Err(e) = delete_response {
                        panic!("{:?}", e);
                    }
                }
            }
        }
    }
}

pub fn start_verify_jobs(database: Database) {
    task::spawn(async move {
        loop {
            key_cleanup(&database).await;
            thread::sleep(Duration::from_millis(THREAD_DELAY));
        }
    });
}
