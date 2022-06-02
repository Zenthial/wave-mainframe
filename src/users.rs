use reqwest::Response;
use std::sync::Mutex;
use tokio::join;

use crate::{
    api_down_queue,
    promotion::{demote, get_required_points, promote, should_demote, should_promote},
    ranks::{Ranks, STRanks, SableRanks},
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Divisions {
    #[serde(default)]
    pub st: Option<STRanks>,
    #[serde(default)]
    pub sable: Option<SableRanks>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    #[serde(default)]
    pub user_id: u64,
    pub name: String,
    #[serde(default)]
    pub points: u64,
    #[serde(default)]
    pub events: u64,
    #[serde(default)]
    pub floor_points: Option<u64>,
    #[serde(default)]
    pub goal_points: Option<u64>,
    #[serde(default)]
    pub rank: Ranks,
    pub divisions: Option<Divisions>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeserializeUser {
    #[serde(default)]
    pub user_id: u64,
    pub name: String,
    #[serde(default)]
    pub points: u64,
    #[serde(default)]
    pub events: u64,
    #[serde(default)]
    pub floor_points: Option<u64>,
    #[serde(default)]
    pub goal_points: Option<u64>,
    #[serde(default)]
    pub rank: String,
    pub divisions: Option<Divisions>,
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

async fn get_ranks(roblox_id: u64) -> (Option<Ranks>, Option<STRanks>, Option<SableRanks>) {
    let (main_group_result, st_result, sable_result) = join!(
        get_rank_in_group(WIJ_ID, roblox_id),
        get_rank_in_group(ST_ID, roblox_id),
        get_rank_in_group(SABLE_ID, roblox_id)
    );

    let main_group_rank = parse_rank(main_group_result);
    let st_rank_option = parse_rank(st_result);
    let sable_rank_option = parse_rank(sable_result);

    let rank = match main_group_rank {
        Some(rank) => Ranks::from_value(rank),
        None => None,
    };

    let st_rank = match st_rank_option {
        Some(rank) => STRanks::from_value(rank),
        None => None,
    };

    let sable_rank = match sable_rank_option {
        Some(rank) => SableRanks::from_value(rank),
        None => None,
    };

    (rank, st_rank, sable_rank)
}

async fn create_user_from_id(roblox_id: u64) -> Option<User> {
    let (user_info_result, ranks) = join!(get_user_info_from_id(roblox_id), get_ranks(roblox_id),);

    let user_info: UsernameResponse = match user_info_result {
        Ok(info) => info,
        Err(e) => panic!("{}", e.to_string()),
    };

    let (rank, st_rank, sable_rank) = ranks;

    if let Some(rank_enum) = rank {
        let mut divisions = None;
        if st_rank.is_some() || sable_rank.is_some() {
            divisions = Some(Divisions {
                st: st_rank,
                sable: sable_rank,
            });
        }

        let goal_points = match rank_enum.get_next() {
            Some(rank) => get_required_points(rank),
            None => None,
        };

        let required_points = get_required_points(rank_enum.clone());

        let user_struct = User {
            user_id: roblox_id,
            name: user_info.name,
            points: {
                if required_points.is_some() {
                    required_points.unwrap()
                } else {
                    0
                }
            },
            floor_points: required_points,
            goal_points,
            rank: rank_enum,
            events: 0,
            divisions,
        };

        return Some(user_struct);
    }

    None
}

async fn reconcile_user(user: &mut User) {
    let (ranks, user_info) = join!(get_ranks(user.user_id), get_user_info_from_id(user.user_id));
    let (rank, st_rank, sable_rank) = ranks;

    if let Some(rank_enum) = rank {
        let mut divisions = None;
        if st_rank.is_some() || sable_rank.is_some() {
            divisions = Some(Divisions {
                st: st_rank,
                sable: sable_rank,
            });
        }

        let goal_points = match rank_enum.get_next() {
            Some(rank) => get_required_points(rank),
            None => None,
        };

        let required_points = get_required_points(rank_enum.clone());

        if required_points.is_some() && user.points < required_points.unwrap() {
            user.points = required_points.unwrap();
            user.floor_points = required_points;
            user.goal_points = goal_points;
        }

        user.divisions = divisions;

        if user_info.is_ok() {
            user.name = user_info.unwrap().name;
        }
    }
}

async fn get_real_user_from_deserialize(response: Response) -> Result<User, reqwest::Error> {
    let d_user = response.json::<DeserializeUser>().await?;

    let rank_enum_option = Ranks::inverse_to_string(d_user.rank);
    let rank_enum = match rank_enum_option {
        Some(r) => r,
        None => Ranks::Enlisted,
    };

    let mut real_user = User {
        user_id: d_user.user_id,
        name: d_user.name,
        points: d_user.points,
        events: d_user.events,
        floor_points: d_user.floor_points,
        goal_points: d_user.goal_points,
        rank: rank_enum,
        divisions: d_user.divisions,
    };

    reconcile_user(&mut real_user).await;

    Ok(real_user)
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
                let json_result = get_real_user_from_deserialize(update_response).await;

                match json_result {
                    Ok(response) => return HttpResponse::Ok().json(response),
                    Err(_) => {
                        let possible_user = create_user_from_id(user_id).await;

                        match possible_user {
                            Some(user) => {
                                let put_response =
                                    put_user(user.user_id, user.clone(), &data.database).await;

                                parse_user_result(put_response).await
                            }
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
            let mut user = get_real_user_from_deserialize(response).await.unwrap();
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
