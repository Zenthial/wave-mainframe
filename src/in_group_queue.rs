use crate::promotion::{demote, log_to_discord, promote, should_demote, should_promote};
use crate::roblox::RobloxAccount;
use crate::users::{reconcile_user, User};
use crate::{api_down_queue, roblox::get_rank_in_group};
use firebase_realtime_database::Database;
use std::{
    collections::{HashMap, LinkedList},
    thread,
    time::Duration,
};
use tokio::task;

static WIJ_ID: u64 = 3747606;
static THREAD_DELAY: u64 = 30000;
static QUEUE_POP_NUM: u64 = 6;

async fn initialize_queue(database: &Database) -> Option<LinkedList<User>> {
    let response = database.get("users/").await;
    if response.is_ok() {
        let hash_map_result = response.unwrap().json::<HashMap<u64, User>>().await;
        if hash_map_result.is_ok() {
            let hash_map = hash_map_result.unwrap();
            let mut queue = LinkedList::<User>::new();

            for (_user_id, user) in hash_map.iter() {
                queue.push_back(user.clone());
            }

            return Some(queue);
        }
    }

    None
}

async fn promotion_checker(
    user: &mut User,
    database: &Database,
    roblox_account: &mut RobloxAccount,
) {
    if !should_promote(&user) {
        if should_demote(&user) {
            let success = demote(user, roblox_account).await;
            if !success {
                api_down_queue::add_demote(database, user).await;
            }
        }
    } else {
        let success = promote(user, roblox_account).await;
        if !success {
            api_down_queue::add_promote(database, user).await;
        }
    }
}

async fn queue_handler(
    database: &Database,
    queue: &mut LinkedList<User>,
    roblox_account: &mut RobloxAccount,
) -> bool {
    for _ in 0..QUEUE_POP_NUM {
        if !queue.is_empty() {
            let user_result = queue.pop_front();

            if user_result.is_none() {
                continue;
            }

            let mut user = user_result.unwrap();

            let main_rank_result = get_rank_in_group(WIJ_ID, user.user_id).await;
            if main_rank_result.is_err() {
                return false;
            }

            let main_rank_option = main_rank_result.unwrap();
            if main_rank_option.is_none() {
                let database_response = database
                    .delete(format!("users/{}", user.user_id).as_str())
                    .await;

                if database_response.is_err() {
                    println!("{:?}", database_response);
                } else {
                    log_to_discord(format!("deleted {}", user.user_id)).await;
                }
            } else {
                promotion_checker(&mut user, database, roblox_account).await;
                reconcile_user(&mut user, database).await;
            }
        } else {
            return false;
        }
    }

    return true;
}

pub fn start_queue_jobs(database: Database, mut roblox_account: RobloxAccount) {
    task::spawn(async move {
        let queue_result = initialize_queue(&database).await;

        if queue_result.is_some() {
            let mut queue: LinkedList<User> = queue_result.unwrap();
            loop {
                if queue_handler(&database, &mut queue, &mut roblox_account).await == false {
                    start_queue_jobs(database, roblox_account);
                    break;
                }
                thread::sleep(Duration::from_millis(THREAD_DELAY));
            }
        }
    });
}
