use crate::roblox::get_user_info_from_id;
use firebase_realtime_database::{Database, FirebaseError};
use tokio::join;

use crate::definitions::ranks::{Ranks, STRanks, SableRanks};
use crate::definitions::users_definitions::{Divisions, User, SABLE_ID, ST_ID, WIJ_ID};
use crate::functions::promotion::get_required_points;
use crate::roblox::get_rank_in_group;

pub async fn put_user(
    user_id: u64,
    database_user: User,
    database: &Database,
) -> Result<reqwest::Response, FirebaseError> {
    database
        .put(format!("users/{}", user_id).as_str(), &database_user)
        .await
}

fn parse_rank(rank_result: Result<Option<u64>, reqwest::Error>) -> Option<u64> {
    match rank_result {
        Ok(rank) => rank,
        Err(e) => panic!("{}", e.to_string()),
    }
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
        println!("{:?} {:?}", goal_points, required_points);

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

        let _create_result = put_user(user.user_id, user.clone(), database).await;
    }
}
