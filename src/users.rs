use crate::{ranks::Ranks, AppState};
use actix_web::{
    delete, get, patch, put,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use firebase_realtime_database::FirebaseError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    #[serde(default)]
    points: u64,
    #[serde(default)]
    rank: Ranks,
}

async fn parse_user_result(result: Result<reqwest::Response, FirebaseError>) -> HttpResponse {
    match result {
        Ok(update_response) => {
            let status_code = update_response.status();
            if status_code == 200 {
                let json_result = update_response.json::<User>().await;

                match json_result {
                    Ok(response) => return HttpResponse::Ok().json(response),
                    Err(e) => {
                        return HttpResponse::InternalServerError().body(format!(
                            "Failed to deserialize database json body, got error {}",
                            e
                        ))
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

#[delete("users/{user_id}")]
async fn delete_user(path: Path<u64>, data: Data<AppState>) -> HttpResponse {
    let user_id = path.into_inner();
    let delete_result = data
        .database
        .delete(format!("users/{}", user_id).as_str())
        .await;

    match delete_result {
        Ok(update_response) => {
            let status_code = update_response.status();
            if status_code == 200 {
                let json_result = update_response.text().await;

                match json_result {
                    Ok(response) => return HttpResponse::Ok().body(response),
                    Err(e) => {
                        return HttpResponse::InternalServerError().body(format!(
                            "Failed to deserialize database json body, got error {}",
                            e
                        ))
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

#[put("users/{user_id}")]
async fn create_user(path: Path<u64>, user: Json<User>, data: Data<AppState>) -> HttpResponse {
    let user_id = path.into_inner();
    let create_result = data
        .database
        .put(format!("users/{}", user_id).as_str(), &user)
        .await;

    parse_user_result(create_result).await
}

#[patch("users/{user_id}")]
async fn update_user(path: Path<u64>, user: Json<User>, data: Data<AppState>) -> HttpResponse {
    let user_id = path.into_inner();
    let update_result = data
        .database
        .update(format!("users/{}", user_id).as_str(), &user)
        .await;

    parse_user_result(update_result).await
}

#[get("users/{user_id}")]
async fn get_user(path: Path<u64>, data: Data<AppState>) -> HttpResponse {
    let user_id = path.into_inner();
    let user_result = data
        .database
        .get(format!("users/{}", user_id).as_str())
        .await;

    parse_user_result(user_result).await
}

pub fn configure_users(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(update_user);
    cfg.service(create_user);
    cfg.service(delete_user);
}
