use crate::{ranks::Ranks, roblox::RobloxAccount, users::User};
use reqwest::Client;
use serde::Serialize;

static WEBHOOK_URL: &'static str = "https://discord.com/api/webhooks/662771092292632627/Rzk9gbQTEaN6EoEhv2VHgxJihRJ4t-9PQAoSOkhoZP_fbTs8dQjHp9AfZxOENjy8uQZo";
static WIJ_ID: u64 = 3747606;

pub fn get_required_points(rank: Ranks) -> Option<u64> {
    match rank {
        Ranks::StaffSergeant => Some(900),
        Ranks::TechSergeant => Some(625),
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

#[derive(Serialize, Debug)]
struct WebhookBody {
    content: String,
}

async fn log_to_discord(message: String) {
    let client = Client::new();

    let _response = client
        .post(WEBHOOK_URL)
        .json(&WebhookBody { content: message })
        .send()
        .await;
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
        .set_rank(user.user_id.try_into().unwrap(), WIJ_ID, user.rank.clone())
        .await;

    match result {
        Ok(b) => {
            log_to_discord(format!("promoted user {}-{}", user.user_id, user.name)).await;
            return b;
        }
        Err(e) => {
            log_to_discord(format!("ERROR: {}", e.to_string())).await;
            panic!("{}", e.to_string());
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
        .set_rank(user.user_id.try_into().unwrap(), WIJ_ID, user.rank.clone())
        .await;

    match result {
        Ok(b) => {
            log_to_discord(format!("demoted user {}-{}", user.user_id, user.name)).await;
            return b;
        }
        Err(e) => {
            log_to_discord(format!("ERROR: {}", e.to_string())).await;
            panic!("{}", e.to_string());
        }
    }
}
