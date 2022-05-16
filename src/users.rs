use std::sync::Mutex;
use tokio::join;

use crate::{
    api_down_queue,
    promotion::{demote, promote, should_demote, should_promote},
    ranks::Ranks,
    roblox::{get_rank_in_group, get_user_info_from_id, UsernameResponse},
    AppState,
};
use actix_web::{
    delete, get, patch, post, put,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use firebase_realtime_database::{Database, FirebaseError};
use serde::{Deserialize, Serialize};

static WIJ_ID: u64 = 3747606;
static ST_ID: u64 = 3758883;
static SABLE_ID: u64 = 5430057;

#[derive(Deserialize, Serialize, Debug)]
pub struct Divisions {
    #[serde(default)]
    pub st: bool,
    #[serde(default)]
    pub sable: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(default)]
    pub user_id: u64,
    pub name: String,
    #[serde(default)]
    pub points: u64,
    #[serde(default)]
    pub rank: Ranks,
    pub divisions: Divisions,
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

pub async fn put_user(
    user_id: u64,
    database_user: User,
    database: &Database,
) -> Result<reqwest::Response, FirebaseError> {
    database
        .put(format!("users/{}", user_id).as_str(), &database_user)
        .await
}

#[put("users/{user_id}")]
async fn create_user(
    path: Path<u64>,
    user: Json<User>,
    mutex: Data<Mutex<AppState>>,
) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let create_result = put_user(user_id, user.into_inner(), &data.database).await;

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

fn parse_rank(rank_result: Result<Option<u64>, reqwest::Error>) -> Option<u64> {
    match rank_result {
        Ok(rank) => rank,
        Err(e) => panic!("{}", e.to_string()),
    }
}

async fn create_user_from_id(roblox_id: u64) -> Option<User> {
    let (user_info_result, main_group_result, st_result, sable_result) = join!(
        get_user_info_from_id(roblox_id),
        get_rank_in_group(WIJ_ID, roblox_id),
        get_rank_in_group(ST_ID, roblox_id),
        get_rank_in_group(SABLE_ID, roblox_id)
    );

    let user_info: UsernameResponse = match user_info_result {
        Ok(info) => info,
        Err(e) => panic!("{}", e.to_string()),
    };

    let main_group_rank = parse_rank(main_group_result);
    let st_rank = parse_rank(st_result);
    let sable_rank = parse_rank(sable_result);

    let rank = match main_group_rank {
        Some(rank) => Ranks::from_value(rank),
        None => None,
    };

    if let Some(rank_enum) = rank {
        let user_struct = User {
            user_id: roblox_id,
            name: user_info.username,
            points: 0,
            rank: rank_enum,
            divisions: Divisions {
                st: st_rank.is_some(),
                sable: sable_rank.is_some(),
            },
        };

        return Some(user_struct);
    }

    None
}

#[get("users/{user_id}")]
async fn get_user(path: Path<u64>, mutex: Data<Mutex<AppState>>) -> HttpResponse {
    let data = mutex.lock().unwrap();

    let user_id = path.into_inner();
    let user_result = data
        .database
        .get(format!("users/{}", user_id).as_str())
        .await;

    match user_result {
        Ok(update_response) => {
            let status_code = update_response.status();
            if status_code == 200 {
                let json_result = update_response.json::<User>().await;

                match json_result {
                    Ok(response) => return HttpResponse::Ok().json(response),
                    Err(_) => {
                        let possible_user = create_user_from_id(user_id).await;

                        match possible_user {
                            Some(user) => HttpResponse::Ok().json(&user),
                            None => HttpResponse::InternalServerError().body("User is not in WIJ"),
                        }
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
