use std::sync::Mutex;

use crate::{
    api_down_queue,
    promotion::{demote, promote, should_demote, should_promote},
    ranks::Ranks,
    AppState,
};
use actix_web::{
    delete, get, patch, post, put,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use firebase_realtime_database::FirebaseError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(default)]
    pub user_id: i32,
    pub name: String,
    #[serde(default)]
    pub points: u64,
    #[serde(default)]
    pub rank: Ranks,
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
async fn delete_user(path: Path<u64>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

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
async fn create_user(
    path: Path<u64>,
    user: Json<User>,
    mutex: Data<Mutex<AppState>>,
) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let create_result = data
        .database
        .put(format!("users/{}", user_id).as_str(), &user)
        .await;

    parse_user_result(create_result).await
}

#[patch("users/{user_id}")]
async fn update_user(
    path: Path<u64>,
    user: Json<User>,
    mutex: Data<Mutex<AppState>>,
) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let update_result = data
        .database
        .update(format!("users/{}", user_id).as_str(), &user)
        .await;

    parse_user_result(update_result).await
}

#[get("users/{user_id}")]
async fn get_user(path: Path<u64>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let user_result = data
        .database
        .get(format!("users/{}", user_id).as_str())
        .await;

    parse_user_result(user_result).await
}

#[derive(Deserialize)]
struct PointsBody {
    points_to_add: u64,
}

#[post("users/{user_id}")]
async fn add_points(
    path: Path<u64>,
    body: Json<PointsBody>,
    mutex: Data<Mutex<AppState>>,
) -> HttpResponse {
    let mut data = mutex.lock().unwrap();
    let user_id = path.into_inner();
    let user_result = data
        .database
        .get(format!("users/{}", user_id).as_str())
        .await;

    match user_result {
        Ok(response) => {
            let mut user = response.json::<User>().await.unwrap();
            user.points += body.points_to_add;

            if !should_promote(&user) {
                if should_demote(&user) {
                    let success = demote(&mut user, &mut data.roblox_user).await;
                    if !success {
                        api_down_queue::add_demote(&data.database, &user).await;
                    }
                }
            } else {
                let success = promote(&mut user, &mut data.roblox_user).await;
                if !success {
                    api_down_queue::add_promote(&data.database, &user).await;
                }
            }

            let update_result = data
                .database
                .update(format!("users/{}", user_id).as_str(), &user)
                .await;

            return parse_user_result(update_result).await;
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.message),
    }
}

pub fn configure_users(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(update_user);
    cfg.service(create_user);
    cfg.service(delete_user);
    cfg.service(add_points);
}
