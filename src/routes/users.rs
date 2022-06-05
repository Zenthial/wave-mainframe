use reqwest::Response;
use std::sync::Mutex;
use tokio::join;

use crate::{
    definitions::users_definitions::{Divisions, User},
    definitions::{ranks::Ranks, users_definitions::DeserializeUser},
    functions::promotion::{demote, get_required_points, promote, should_demote, should_promote},
    functions::users_functions::{get_ranks, put_user, reconcile_user},
    jobs::api_down_queue,
    logs::log_to_discord,
    roblox::{get_user_info_from_id, UsernameResponse},
    AppState,
};
use actix_web::{
    delete, get, patch, post, put,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use firebase_realtime_database::{Database, FirebaseError};
use serde::Deserialize;

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

async fn create_user_from_id(roblox_id: u64) -> Option<User> {
    let (user_info_result, ranks) = join!(get_user_info_from_id(roblox_id), get_ranks(roblox_id),);

    let user_info: UsernameResponse = match user_info_result {
        Ok(info) => info,
        Err(_) => return None,
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

async fn get_real_user_from_deserialize(
    response: Response,
    database: &Database,
) -> Result<User, reqwest::Error> {
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

    reconcile_user(&mut real_user, database).await;

    println!("{:?}", real_user);
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
                let json_result =
                    get_real_user_from_deserialize(update_response, &data.database).await;

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
    #[serde(default)]
    event: bool,
}

#[post("users/add/{user_id}")]
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
            let json_result = get_real_user_from_deserialize(response, &data.database).await;

            match json_result {
                Ok(mut user) => {
                    if body.event {
                        user.events += 1;
                    }
                    user.points += body.points_to_add;
                    log_to_discord(format!(
                        "Adding {} bP to {} - {}",
                        body.points_to_add, user.user_id, user.name
                    ))
                    .await;

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
                Err(_) => {
                    let possible_user = create_user_from_id(user_id).await;

                    match possible_user {
                        Some(mut user) => {
                            if body.event {
                                user.events += 1;
                            }
                            user.points += body.points_to_add;
                            log_to_discord(format!(
                                "Adding {} bP to {} - {}",
                                body.points_to_add, user.user_id, user.name
                            ))
                            .await;
                            let put_response =
                                put_user(user.user_id, user.clone(), &data.database).await;

                            parse_user_result(put_response).await
                        }
                        None => HttpResponse::BadRequest().body("User is not in WIJ"),
                    }
                }
            }
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.message),
    }
}

#[derive(Deserialize)]
struct RemoveBody {
    points_to_remove: u64,
}

#[post("users/remove/{user_id}")]
async fn remove_points(
    path: Path<u64>,
    body: Json<RemoveBody>,
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
            let json_result = get_real_user_from_deserialize(response, &data.database).await;

            match json_result {
                Ok(mut user) => {
                    user.points -= body.points_to_remove;
                    log_to_discord(format!(
                        "Removing {} bP from {} - {}",
                        body.points_to_remove, user.user_id, user.name
                    ))
                    .await;

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
                Err(_) => HttpResponse::BadRequest().body("User does not have a profile"),
            }
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
    cfg.service(remove_points);
}
