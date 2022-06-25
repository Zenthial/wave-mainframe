use firebase_realtime_database::Database;

use crate::{
    definitions::users_definitions::User, functions::promotion::demote,
    functions::promotion::promote, logs::log_error, roblox::RobloxAccount,
};
use std::collections::HashMap;

pub async fn add_promote(database: &Database, user: &User) {
    let result = database.post("queue/promote", &user).await;
    match result {
        Ok(response) => {
            let status = response.status();
            if status == 400 || status == 401 || status == 404 || status == 500 || status == 503 {
                log_error(format!("{}", response.text().await.unwrap())).await;
            }
        }
        Err(e) => log_error(format!("{}", e.message)).await,
    }
}

pub async fn add_demote(database: &Database, user: &User) {
    let result = database.post("queue/demote", &user).await;
    match result {
        Ok(response) => {
            let status = response.status();
            if status == 400 || status == 401 || status == 404 || status == 500 || status == 503 {
                log_error(format!("{}", response.text().await.unwrap())).await;
            }
        }
        Err(e) => log_error(format!("{}", e.message)).await,
    }
}

pub async fn queue_handler(database: &Database, roblox_account: &mut RobloxAccount) {
    // check promotion
    let promote_result = database.get("queue/promote").await;

    if let Ok(response) = promote_result {
        let user_map_option = response
            .json::<Option<HashMap<String, User>>>()
            .await
            .unwrap();

        if let Some(user_map) = user_map_option {
            for (key, mut user) in user_map {
                let delete_response = database
                    .delete(format!("queue/promote/{}", key).as_str())
                    .await;

                if let Ok(_) = delete_response {
                    promote(&mut user, roblox_account).await;
                }
            }
        }
    }

    let demote_result = database.get("queue/demote").await;

    if let Ok(response) = demote_result {
        let user_map_option = response
            .json::<Option<HashMap<String, User>>>()
            .await
            .unwrap();
        if let Some(user_map) = user_map_option {
            for (key, mut user) in user_map {
                let delete_response = database
                    .delete(format!("queue/promote/{}", key).as_str())
                    .await;

                if let Ok(_) = delete_response {
                    demote(&mut user, roblox_account).await;
                }
            }
        }
    }
}