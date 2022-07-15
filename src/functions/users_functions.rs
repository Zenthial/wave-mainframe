use crate::roblox::{get_user_info_from_id, UsernameResponse};
use firebase_realtime_database::Database;
use tokio::join;

use crate::definitions::ranks::{Ranks, STRanks, SableRanks};
use crate::definitions::users_definitions::{Divisions, User, SABLE_ID, ST_ID, WIJ_ID};
use crate::functions::promotion::get_required_points;
use crate::roblox::get_rank_in_group;

fn parse_rank(rank_result: Result<Option<u64>, reqwest::Error>) -> Option<u64> {
    match rank_result {
        Ok(rank) => rank,
        Err(_) => {
            return None;
        }
    }
}

pub async fn _create_user_from_id(roblox_id: u64) -> Option<User> {
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

pub async fn get_ranks(roblox_id: u64) -> (Option<Ranks>, Option<STRanks>, Option<SableRanks>) {
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

pub async fn reconcile_user(user: &mut User, database: &Database) {
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
        }

        user.floor_points = required_points;
        user.goal_points = goal_points;
        user.rank = rank_enum;
        user.divisions = divisions;

        if user_info.is_ok() {
            user.name = user_info.unwrap().name;
        }

        let _create_result = database
            .put(format!("users/{}", user.user_id).as_str(), &user)
            .await;
    }
}
