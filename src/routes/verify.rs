use std::time::SystemTime;

use actix_web::{
    get, post, put,
    web::{Data, Json, Path, ServiceConfig},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    functions::verify_functions::{
        get_verification_body, is_verified, VerificationBody, VerifiedStruct,
    },
    AppState,
};

#[derive(Deserialize)]
struct Verification {
    discord_id: String,
    roblox_username: String,
}

/// Places a discord user in the verify/awaiting section
/// Discord user provides their roblox username
/// Roblox user joins the game then and the two are linked together
#[put("verify/")]
async fn request_verification(body: Json<Verification>, app_state: Data<AppState>) -> HttpResponse {
    let database = &app_state.database;

    let verification_body = VerificationBody {
        discord_id: body.discord_id.clone(),
        creation_time: SystemTime::now(),
    };

    let verification_create_result = database
        .put(
            format!("verification/awaiting/{}", body.roblox_username).as_str(),
            &verification_body,
        )
        .await;

    match verification_create_result {
        Ok(response) => HttpResponse::Ok().body(response.text().await.unwrap()),
        Err(e) => HttpResponse::InternalServerError().body(e.message),
    }
}

#[derive(Deserialize)]
struct RobloxVerification {
    username: String,
    user_id: u32,
}

/// Checks to see if a user who joined the roblox game is looking to be verified
/// If they are, they are moved from the awaiting to the verified section
/// Their roblox userid is logged
#[post("verify/")]
async fn check_verification(
    body: Json<RobloxVerification>,
    app_state: Data<AppState>,
) -> HttpResponse {
    let database = &app_state.database;

    let verification_option = get_verification_body::<VerificationBody>(
        format!("verification/awaiting/{}", body.username).as_str(),
        database,
    )
    .await;
    if verification_option.is_none() {
        return HttpResponse::InternalServerError()
            .body("Firebase failed to read verification database");
    }

    let verification_body = verification_option.unwrap();
    let verified_struct = VerifiedStruct {
        roblox_id: body.user_id,
        discord_id: verification_body.discord_id.clone(),
    };
    let _put_result = database
        .put(
            format!("verification/discord/{}", verification_body.discord_id).as_str(),
            &verified_struct,
        )
        .await;

    HttpResponse::Ok().body("Success!")
}

/// Verification checker
/// Gets the verification struct from the discord userid
#[get("verify/{discord_id}")]
async fn get_verification(path: Path<String>, app_state: Data<AppState>) -> HttpResponse {
    let discord_user_id = path.into_inner();
    let database = &app_state.database;

    let verification_option = is_verified(discord_user_id, database).await;
    if verification_option.is_none() {
        return HttpResponse::InternalServerError()
            .body("Firebase failed to read verification database");
    }

    let verified_struct = verification_option.unwrap();
    HttpResponse::Ok().json(verified_struct)
}

pub fn configure_verify(cfg: &mut ServiceConfig) {
    cfg.service(request_verification);
    cfg.service(check_verification);
    cfg.service(get_verification);
}
