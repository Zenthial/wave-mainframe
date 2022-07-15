use reqwest::Response;
use serde::Deserialize;

use crate::{
    definitions::users_definitions::User,
    definitions::{ranks::Ranks, users_definitions::DeserializeUser},
    functions::{promotion::check_promotion, users_functions::reconcile_user},
    logs::log_error,
    AppState,
};
use actix_web::{
    get, post, put,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use firebase_realtime_database::Database;

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

#[put("users/{user_id}")]
async fn create_user(path: Path<u32>, user: Json<User>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database;

    let user_id = path.into_inner();
    let create_result = database
        .put(format!("users/{}", user_id).as_str(), &user)
        .await;

    if create_result.is_err() {
        let err_str = create_result.unwrap_err();
        return HttpResponse::InternalServerError().json(err_str.message);
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
async fn new_get_user(path: Path<u32>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database;

    let user_id = path.into_inner();
    let user_option = get_user_struct(user_id, database).await;

    match user_option {
        Some(user) => return HttpResponse::Ok().json(user),
        None => return HttpResponse::BadRequest().body(format!("No user found for {}", user_id)),
    }
}

#[derive(Deserialize)]
struct PointUser {
    user_id: u32,
    increment: i32,
}

#[derive(Deserialize)]
struct PointsStruct {
    users: Vec<PointUser>,
}

#[post("users/")]
async fn increment_points(body: Json<PointsStruct>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database;
    let roblox_user_result = app_state.roblox_user.write();
    if roblox_user_result.is_err() {
        let err = roblox_user_result.unwrap_err();

        log_error(format!("Poison error in RwLock: {}", err.to_string())).await;
        return HttpResponse::InternalServerError().body("Internal poison error");
    }
    let mut roblox_user = roblox_user_result.unwrap();

    let succeed_vec: Vec<u32> = vec![];
    for user in body.users.iter() {
        let user_option = get_user_struct(user.user_id, database).await;

        if user_option.is_some() {
            let mut user_struct = user_option.unwrap();

            let mut points = user_struct.points as i32;
            points += user.increment;

            user_struct.points += points as u64;

            let _create_result = database
                .put(
                    format!("users/{}", user_struct.user_id).as_str(),
                    &user_struct,
                )
                .await;

            check_promotion(&mut user_struct, database, &mut roblox_user).await;
        }
    }

    HttpResponse::Ok().json(succeed_vec)
}

pub fn configure_users(cfg: &mut web::ServiceConfig) {
    cfg.service(new_get_user);
    cfg.service(create_user);
}
