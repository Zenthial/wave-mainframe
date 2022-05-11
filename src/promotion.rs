use crate::{ranks::Ranks, roblox::RobloxAccount, users::User};

static WIJ_ID: i32 = 3747606;

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
            println!("promoted user {}-{}", user.user_id, user.name);
            return b;
        }
        Err(e) => panic!("{}", e.to_string()),
    }
}

pub async fn demote(user: &mut User, roblox_account: &mut RobloxAccount) -> bool {
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
            println!("demoted user {}-{}", user.user_id, user.name);
            return b;
        }
        Err(e) => panic!("{}", e.to_string()),
    }
}
