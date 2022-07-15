use std::time::SystemTime;

use firebase_realtime_database::Database;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationBody {
    pub discord_id: String,
    pub creation_time: SystemTime,
}

pub async fn get_verification_body<T: DeserializeOwned>(
    path: &str,
    database: &Database,
) -> Option<T> {
    let verification_result = database.get(path).await;
    if verification_result.is_err() {
        return None;
    }

    let verification_response = verification_result.unwrap();
    let verification_body_result = verification_response.json::<T>().await;
    if verification_body_result.is_err() {
        return None;
    }

    let verification_struct = verification_body_result.unwrap();
    Some(verification_struct)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VerifiedStruct {
    pub roblox_user_id: u32,
    pub discord_id: String,
}

pub async fn is_verified(discord_user_id: String, database: &Database) -> Option<VerifiedStruct> {
    get_verification_body::<VerifiedStruct>(
        format!("verify/verified/{}", discord_user_id).as_str(),
        database,
    )
    .await
}
