use std::time::{Duration, SystemTime};

use firebase_realtime_database::Database;

use crate::{functions::verify_functions::get_verification_map, logs::log_error};

static CLEANUP_TIMEOUT: u64 = 60000 * 5; // five minutes

pub async fn key_cleanup(db: &Database) {
    let user_map_option = get_verification_map(&db).await;

    if let Some(user_map) = user_map_option {
        for (key, user) in user_map {
            let current_time = SystemTime::now();
            let time_between_bool = current_time.duration_since(user.creation_time).unwrap()
                >= Duration::from_millis(CLEANUP_TIMEOUT);
            if time_between_bool {
                let delete_response = db
                    .delete(format!("verification/awaiting/{}", key).as_str())
                    .await;

                if let Err(e) = delete_response {
                    log_error(format!("{:?}", e)).await;
                } else if let Ok(_) = delete_response {
                    println!("deleted code for user {}", user.discord_id);
                }
            }
        }
    }
}
