use std::{collections::HashMap, time::SystemTime};

use firebase_realtime_database::Database;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationCodeBody {
    pub discord_id: String,
    pub creation_time: SystemTime,
}

pub async fn get_verification_map(
    database: &Database,
) -> Option<HashMap<String, VerificationCodeBody>> {
    let awaiting_result = database.get("verification/awaiting/").await;

    if let Ok(response) = awaiting_result {
        let user_map_option = response
            .json::<Option<HashMap<String, VerificationCodeBody>>>()
            .await
            .unwrap();

        return user_map_option;
    }

    None
}
