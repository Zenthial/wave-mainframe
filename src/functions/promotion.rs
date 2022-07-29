use firebase_realtime_database::Database;

use crate::{
    definitions::ranks::Ranks,
    definitions::users::User,
    logs::{log_error, log_to_discord},
    roblox::RobloxAccount,
};

use super::users::reconcile_user;

static WIJ_ID: u32 = 3747606;

pub fn get_required_points(rank: Ranks) -> Option<i32> {
    match rank {
        Ranks::StaffSergeant => Some(900),
        Ranks::TechSergeant => Some(600),
        Ranks::Corporal => Some(325),
        Ranks::LanceCorporal => Some(270),
        Ranks::Sentinel => Some(190),
        Ranks::Fleetman => Some(115),
        Ranks::Specialist => Some(65),
        Ranks::Operative => Some(30),
        Ranks::Trooper => Some(10),
        Ranks::Enlisted => Some(0),
        _ => None,
    }
}

pub fn should_promote(user: &User) -> bool {
    let next_rank = if let Some(rank) = user.rank.get_next() {
        rank
    } else {
        return false;
    };

    let promotion_points = get_required_points(next_rank);
    match promotion_points {
        Some(points) => return user.points >= points,
        None => return false,
    }
}

pub fn should_demote(user: &User) -> bool {
    let prev_rank = if let Some(rank) = user.rank.get_prev() {
        rank
    } else {
        return false;
    };

    let demotion_points = get_required_points(prev_rank);
    match demotion_points {
        Some(points) => return user.points < points,
        None => return false,
    }
}

pub async fn promote(user: &mut User, roblox_account: &mut RobloxAccount) -> bool {
    if !should_promote(user) {
        return false;
    }

    let next_rank = if let Some(rank) = user.rank.get_next() {
        rank
    } else {
        return false;
    };

    user.rank = next_rank;

    let result = roblox_account
        .set_rank(user.user_id, WIJ_ID, user.rank.clone())
        .await;

    match result {
        Ok(b) => {
            log_to_discord(format!("Promoted user {} - {}", user.user_id, user.name)).await;
            log_error(format!(
                "**Promoted** user {} - {}",
                user.user_id, user.name
            ))
            .await;
            return b;
        }
        Err(e) => {
            log_error(format!("ERROR: {}", e.to_string())).await;
            return false;
        }
    }
}

pub async fn demote(user: &mut User, roblox_account: &mut RobloxAccount) -> bool {
    if !should_demote(user) {
        return false;
    }

    let prev_rank = if let Some(rank) = user.rank.get_prev() {
        rank
    } else {
        return false;
    };

    user.rank = prev_rank;

    let result = roblox_account
        .set_rank(user.user_id, WIJ_ID, user.rank.clone())
        .await;

    match result {
        Ok(b) => {
            log_to_discord(format!("Demoted user {}-{}", user.user_id, user.name)).await;
            log_error(format!("**Demoted** user {} - {}", user.user_id, user.name)).await;
            return b;
        }
        Err(e) => {
            log_error(format!("ERROR: {}", e.to_string())).await;
            return false;
        }
    }
}

pub async fn check_promotion(
    user: &mut User,
    database: &Database,
    roblox_account: &mut RobloxAccount,
) {
    if should_promote(user) {
        promote(user, roblox_account).await;
        reconcile_user(user, database).await;
    } else if should_demote(user) {
        demote(user, roblox_account).await;
        reconcile_user(user, database).await;
    }
}
