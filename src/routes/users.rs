use std::collections::HashMap;

use log::info;
use reqwest::Response;
use serde::Deserialize;

use crate::{
    definitions::users::{BPLog, User},
    definitions::{ranks::Ranks, users::DeserializeUser},
    functions::{
        promotion::check_promotion,
        users::{self, reconcile_user},
    },
    logs::{log_error, log_to_discord},
    roblox::get_user_ids_from_usernames,
    AppState,
};
use actix_web::{
    get, post, put,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use firebase_realtime_database::{Database, FirebaseError};

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
        total_points: d_user.total_points,
        events: d_user.events,
        floor_points: d_user.floor_points,
        goal_points: d_user.goal_points,
        rank: rank_enum,
        prestige: d_user.prestige,
        divisions: d_user.divisions,
        bp_logs: d_user.bp_logs,
    };

    reconcile_user(&mut real_user, database).await;

    println!("{:?}", real_user);
    Ok(real_user)
}

#[put("users/{user_id}")]
async fn create_user(path: Path<u32>, user: Json<User>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database.read();

    let user_id = path.into_inner();
    let create_result = database
        .put(format!("users/{}", user_id).as_str(), &user)
        .await;

    if create_result.is_err() {
        let err = create_result.unwrap_err();
        match err {
            FirebaseError::GcpAuthError(e) => {
                return HttpResponse::InternalServerError().body(format!("{:?}", e))
            }
            FirebaseError::ReqwestError(e) => {
                return HttpResponse::InternalServerError().json(format!("{:?}", e))
            }
        }
    }

    let response = create_result.unwrap();
    let user_struct = response.json::<User>().await;
    match user_struct {
        Ok(user) => {
            log_error(format!("created user {}", user_id)).await;
            return HttpResponse::Ok().json(user);
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

async fn get_user_struct(user_id: u32, database: &Database) -> Option<User> {
    let user_get_result = database.get(format!("users/{}", user_id).as_str()).await;
    if user_get_result.is_err() {
        return None;
    }

    let json_result = get_real_user_from_deserialize(user_get_result.unwrap(), &database).await;
    match json_result {
        Ok(user) => return Some(user),
        Err(_) => return None,
    }
}

#[get("users/{user_id}")]
async fn get_user(path: Path<u32>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database.read();

    let user_id = path.into_inner();
    let user_option = get_user_struct(user_id, database).await;

    match user_option {
        Some(user) => return HttpResponse::Ok().json(user),
        None => {
            let attempted_created_user = users::create_user_from_id(user_id).await;
            info!("{:?}", attempted_created_user);
            if attempted_created_user.is_none() {
                return HttpResponse::BadRequest().body(format!("No user found for {}", user_id));
            }

            let user = attempted_created_user.unwrap();
            info!("{:?}", user);
            let _create_result = database
                .put(format!("users/{}", user_id).as_str(), &user)
                .await;

            return HttpResponse::Ok().json(user);
        }
    }
}

fn handle_bp_logs(
    mut user_struct: User,
    place_name: &Option<String>,
    admin_id: u32,
    increment: i32,
) -> User {
    // handle bp_logs
    if user_struct.bp_logs.is_none() {
        user_struct.bp_logs = Some(vec![]);
    }

    let mut logs = user_struct.bp_logs.unwrap();
    let mut log = BPLog::new(admin_id, increment);
    if place_name.is_some() {
        log.add_place(place_name.as_ref().unwrap())
    }
    logs.push(log);
    user_struct.bp_logs = Some(logs);

    user_struct
}

#[derive(Deserialize, Debug)]
struct PointUser {
    username: String,
    increment: i32,
    add_event: bool,
    admin_id: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PointsStruct {
    users: Vec<PointUser>,
    place_name: Option<String>,
}

#[post("users/points")]
async fn increment_points(body: Json<PointsStruct>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database.read();
    let mut roblox_user = app_state.roblox_user.write();

    if body.users.len() == 0 {
        return HttpResponse::InternalServerError().body("Must supply 1 user");
    }

    let users: HashMap<String, &PointUser> = body
        .users
        .iter()
        .map(|user| (user.username.to_owned().to_lowercase(), user))
        .collect();

    let usernames_vector: Vec<String> = users
        .iter()
        .map(|(_, user)| user.username.clone())
        .collect();

    let user_id_option = get_user_ids_from_usernames(usernames_vector).await;
    if user_id_option.is_err() {
        return HttpResponse::InternalServerError().body("Roblox failed to return user ids");
    }

    let mut succeed_vec: Vec<(String, u32, i32)> = vec![];
    let mut fail_vec: Vec<(String, u32, i32)> = vec![];
    let user_id_vector = user_id_option.unwrap();
    for (username, user_id_option) in user_id_vector {
        let user_points_payload = users.get(&username.to_lowercase()).unwrap();

        if user_id_option.is_none() {
            continue;
        }

        let user_id = user_id_option.unwrap();
        let user_option = get_user_struct(user_id, database).await;
        if user_option.is_some() {
            let mut user_struct = user_option.unwrap();

            if user_points_payload.add_event {
                user_struct.events += 1;
            }

            user_struct.points += user_points_payload.increment;

            // handle bp_logs
            if user_struct.bp_logs.is_none() {
                user_struct.bp_logs = Some(vec![]);
            }

            user_struct = handle_bp_logs(
                user_struct,
                &body.place_name,
                user_points_payload.admin_id,
                user_points_payload.increment,
            );

            log_to_discord(format!(
                "Adding {} bP to {} - {}",
                user_points_payload.increment, user_struct.user_id, user_struct.name
            ))
            .await;

            let _create_result = database
                .put(
                    format!("users/{}", user_struct.user_id).as_str(),
                    &user_struct,
                )
                .await;

            check_promotion(&mut user_struct, database, &mut roblox_user).await;
            succeed_vec.push((username, user_id, user_points_payload.increment));
        } else {
            let attempted_created_user = users::create_user_from_id(user_id).await;
            if attempted_created_user.is_none() {
                log_to_discord(format!(
                    "Failed to give {} bP to {} - {}.\nUser may need to /wij-verify or join WIJ",
                    user_points_payload.increment, user_id, username
                ))
                .await;
                fail_vec.push((username, user_id, user_points_payload.increment));
                continue;
            }

            let mut user_struct = attempted_created_user.unwrap();
            user_struct.points += user_points_payload.increment;

            user_struct = handle_bp_logs(
                user_struct,
                &body.place_name,
                user_points_payload.admin_id,
                user_points_payload.increment,
            );

            log_to_discord(format!(
                "Adding {} bP to {} - {}",
                user_points_payload.increment, user_struct.user_id, user_struct.name
            ))
            .await;
            let _create_result = database
                .put(format!("users/{}", user_id).as_str(), &user_struct)
                .await;

            check_promotion(&mut user_struct, database, &mut roblox_user).await;
            succeed_vec.push((username, user_id, user_points_payload.increment));
        }
    }

    let mut ok_string: String = succeed_vec
        .iter()
        .map(|(username, user_id, increment)| {
            format!("Added {} bP to {} - {}\n", increment, user_id, username)
        })
        .collect();

    for (username, user_id, increment) in fail_vec {
        ok_string += &format!(
            "Failed to give {} bP to {} - {}.\nUser may need to /wij-verify",
            increment, user_id, username
        );
    }

    ok_string += "";

    HttpResponse::Ok().json(ok_string)
}

pub fn configure_users(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(create_user);
    cfg.service(increment_points);
}
