use std::{sync::Mutex, time::SystemTime};

use actix_web::{
    get, post, put,
    web::{Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use firebase_realtime_database::{Database, FirebaseError};
use serde::{Deserialize, Serialize};

use crate::{functions::verify_functions::VerificationCodeBody, logs::log_error, AppState};

#[derive(Deserialize, Serialize, Debug)]
struct User {
    discord_id: String,
    roblox_id: u64,
}

#[derive(Deserialize, Serialize)]
struct CodeReturnBody {
    code: String,
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
                log_error(format!("Database returned status code {}", status_code)).await;
                return HttpResponse::InternalServerError()
                    .body(format!("Database returned status code {}", status_code));
            }
        }
        Err(e) => {
            log_error(format!("ERROR: {}", e.message)).await;
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

#[derive(Deserialize, Serialize, Debug)]
struct Body {
    discord_id: String,
    username: String,
}

#[put("verify/")]
async fn put_user(body: Json<Body>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let discord_id = &body.discord_id;
    let username = &body.username;

    let put_result = data
        .database
        .put(
            format!("verification/awaiting/{}", username).as_str(),
            &VerificationCodeBody {
                discord_id: discord_id.to_string(),
                creation_time: SystemTime::now(),
            },
        )
        .await;

    match put_result {
        Ok(response) => {
            println!("response: {:?}", response);
            HttpResponse::Ok().body(format!("temporarily linked {} to {}", discord_id, username))
        }
        Err(e) => {
            log_error(format!("error: {:?}", e)).await;
            HttpResponse::InternalServerError().json(e.message)
        }
    }
}

pub async fn create_and_insert_user(
    discord_id: String,
    roblox_id: u64,
    database: &Database,
) -> Result<reqwest::Response, FirebaseError> {
    let user_result = database
        .put(
            format!("verification/discord/{}", discord_id).as_str(),
            &User {
                discord_id,
                roblox_id,
            },
        )
        .await;

    user_result
}

#[derive(Deserialize)]
struct CodePostBody {
    username: String,
    user_id: u64,
}

#[post("verify/")]
async fn check_user(body: Json<CodePostBody>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let code_response = data
        .database
        .get(format!("verification/awaiting/{}", body.username).as_str())
        .await;

    let res = match code_response {
        Ok(response) => {
            let text_result = response.text().await;

            match text_result {
                Ok(text) => Ok(text),
                Err(e) => Err(e.to_string()),
            }
        }
        Err(e) => Err(e.message),
    };

    match res {
        Ok(text) => {
            if text == "null" {
                HttpResponse::NotFound().body("null")
            } else {
                let info_result = serde_json::from_str::<VerificationCodeBody>(&text);
                match info_result {
                    Ok(user) => {
                        let result =
                            create_and_insert_user(user.discord_id, body.user_id, &data.database)
                                .await;

                        match result {
                            Ok(response) => {
                                let verification_user = response.json::<User>().await;
                                match verification_user {
                                    Ok(user) => HttpResponse::Ok().json(&user),
                                    Err(e) => {
                                        HttpResponse::InternalServerError().body(e.to_string())
                                    }
                                }
                            }
                            Err(e) => HttpResponse::InternalServerError().body(e.message),
                        }
                    }
                    Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
                }
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

pub fn configure_verify(cfg: &mut ServiceConfig) {
    cfg.service(get_discord_user);
    cfg.service(put_user);
    cfg.service(check_user);
}
