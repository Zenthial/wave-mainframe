use std::{collections::HashMap, sync::Mutex, time::SystemTime};

use actix_web::{
    get,
    web::{Data, Path, ServiceConfig},
    HttpResponse,
};
use firebase_realtime_database::{Database, FirebaseError};
use serde::{Deserialize, Serialize};

use crate::{key_generation::get_key, AppState};

#[derive(Deserialize, Serialize, Debug)]
struct User {
    discord_id: String,
    roblox_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerificationCodeBody {
    pub discord_id: String,
    pub creation_time: SystemTime,
}

#[derive(Deserialize, Serialize)]
struct CodeReturnBody {
    code: String,
}

pub async fn get_verification_map(
    database: &Database,
) -> Option<HashMap<String, VerificationCodeBody>> {
    let key_result = database.get("verification/keys/").await;

    if let Ok(response) = key_result {
        let user_map_option = response
            .json::<Option<HashMap<String, VerificationCodeBody>>>()
            .await
            .unwrap();

        return user_map_option;
    }

    None
}

async fn parse_user_result(result: Result<reqwest::Response, FirebaseError>) -> HttpResponse {
    match result {
        Ok(update_response) => {
            let status_code = update_response.status();
            if status_code == 200 {
                let json_result = update_response.json::<User>().await;

                match json_result {
                    Ok(response) => return HttpResponse::Ok().json(response),
                    Err(_) => {
                        return HttpResponse::Ok().json(User {
                            discord_id: "0".into(),
                            roblox_id: 0,
                        })
                    }
                }
            } else {
                return HttpResponse::InternalServerError()
                    .body(format!("Database returned status code {}", status_code));
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.message);
        }
    }
}

// the user_id here is a discord userid, not a roblox user id
#[get("verify/discord/{user_id}")]
async fn get_discord_user(path: Path<u64>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let user_result = data
        .database
        .get(format!("verification/discord/{}", user_id).as_str())
        .await;

    parse_user_result(user_result).await
}

async fn already_has_code(discord_id: &str, database: &Database) -> Option<String> {
    let user_map_option = get_verification_map(database).await;

    if let Some(user_map) = user_map_option {
        for (key, user) in user_map {
            if user.discord_id == discord_id {
                return Some(key);
            }
        }
    }

    None
}

#[get("verify/code/{user_id}")]
async fn get_code(path: Path<String>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let potential_key = already_has_code(&user_id, &data.database).await;

    if let Some(key) = potential_key {
        HttpResponse::Ok().json(&CodeReturnBody { code: key })
    } else {
        let key = get_key();
        let put_result = data
            .database
            .put(
                format!("verification/keys/{}", key).as_str(),
                &VerificationCodeBody {
                    discord_id: user_id,
                    creation_time: SystemTime::now(),
                },
            )
            .await;

        match put_result {
            Ok(_response) => HttpResponse::Ok().json(&CodeReturnBody { code: key }),
            Err(e) => HttpResponse::InternalServerError().json(e.message),
        }
    }
}

pub fn configure_verify(cfg: &mut ServiceConfig) {
    cfg.service(get_discord_user);
    cfg.service(get_code);
}
