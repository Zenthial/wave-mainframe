#![allow(dead_code)]

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use reqwest;
use serde::{Deserialize, Serialize};

use crate::{definitions::ranks::Ranks, logs::log_error};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UsernameResponse {
    description: String,
    created: String,
    is_banned: bool,
    external_app_display_name: Option<String>,
    pub id: u32,
    pub name: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    id: u32,
    name: String,
    member_count: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleInfo {
    id: u32,
    name: String,
    rank: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroupInfo {
    group: GroupInfo,
    role: RoleInfo,
}

#[derive(Serialize, Deserialize)]
pub struct GroupResponse {
    data: Option<Vec<UserGroupInfo>>,
}

pub async fn get_user_info_from_id(user_id: u32) -> Result<UsernameResponse, reqwest::Error> {
    let response = reqwest::get(format!("https://users.roblox.com/v1/users/{}", user_id)).await?;
    let username_response = response.json::<UsernameResponse>().await?;

    Ok(username_response)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdResponse {
    requested_username: String,
    has_verified_badge: bool,
    pub id: u32,
    name: String,
    display_name: String,
}

#[derive(Deserialize)]
struct UserIdResponsePayload {
    data: Vec<UserIdResponse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserIdFromUsernamePayload {
    usernames: Vec<String>,
    exclude_banned_users: bool,
}

pub async fn get_user_ids_from_usernames(
    usernames: Vec<String>,
) -> Result<HashMap<String, Option<u32>>, reqwest::Error> {
    let payload = UserIdFromUsernamePayload {
        usernames: usernames.clone(),
        exclude_banned_users: true,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://users.roblox.com/v1/usernames/users")
        .json(&payload)
        .send()
        .await?;

    let user_id_response_payload = response.json::<UserIdResponsePayload>().await?;

    let mut user_id_response_hash_map: HashMap<String, Option<u32>> = HashMap::new();

    for name in usernames.iter() {
        let name_clone = name.clone().to_string();
        user_id_response_hash_map.insert(name_clone, None);
    }
    for user_id_response in user_id_response_payload.data.iter() {
        user_id_response_hash_map
            .insert(user_id_response.name.to_owned(), Some(user_id_response.id));
    }

    Ok(user_id_response_hash_map)
}

pub async fn get_rank_in_group(group_id: u32, user_id: u32) -> Result<Option<u32>, reqwest::Error> {
    let response = reqwest::get(format!(
        "https://groups.roblox.com/v2/users/{}/groups/roles",
        user_id
    ))
    .await?;

    let group_response = response.json::<GroupResponse>().await?;
    if group_response.data.is_some() {
        let data = group_response.data.unwrap();
        let index = data
            .iter()
            .position(|group_info| group_info.group.id == group_id);

        if let Some(i) = index {
            Ok(Some(data.get(i).unwrap().role.rank))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[derive(Serialize)]
pub struct LogoutBody {
    session: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRankBody {
    role_id: u32,
}

#[derive(Clone, Debug)]
pub struct RobloxAccount {
    cookie: String,
    headers: HashMap<String, String>,
    token: String,
    last_token_get: Option<Instant>,
}

impl RobloxAccount {
    fn new(cookie: String) -> RobloxAccount {
        RobloxAccount {
            cookie,
            headers: HashMap::new(),
            token: String::new(),
            last_token_get: None,
        }
    }

    async fn get_current_token(&mut self) -> Option<String> {
        let body = LogoutBody {
            session: self.cookie.clone(),
        };

        let client = reqwest::Client::new();
        let response_result = client
            .post("https://auth.roblox.com/v2/logout")
            .header("cookie", format!(".ROBLOSECURITY={};", self.cookie))
            .header("Referer", "https://www.roblox.com")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match response_result {
            Ok(response) => {
                let headers = response.headers();
                if headers.contains_key("x-csrf-token")
                    || headers.contains_key("x-csrf-token".to_uppercase())
                {
                    let token = headers
                        .get("x-csrf-token")
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();

                    self.token = token.clone();
                    self.last_token_get == Some(Instant::now());
                    return Some(token);
                } else {
                    return None;
                }
            }
            Err(e) => {
                log_error(format!("ERROR: {}", e.to_string())).await;
                return None;
            }
        };
    }

    fn add_header(&mut self, header_name: &str, header_value: &str) {
        if self
            .headers
            .insert(header_name.to_string(), header_value.to_string())
            == None
        {
            println!("failed to add header to hashmap");
            self.add_header(header_name, header_value);
        }
    }

    // public methods

    pub async fn set_rank(
        &mut self,
        user_id: u32,
        group_id: u32,
        rank: Ranks,
    ) -> Result<bool, reqwest::Error> {
        let mut token = self.token.clone();
        if token == "" && self.last_token_get == None {
            let potential_csrf_token = self.get_current_token().await;
            match potential_csrf_token {
                Some(t) => token = t,
                None => {
                    log_error(
                        "Failed to retrieve xCRSF token, is the roblox API down?".to_string(),
                    )
                    .await;
                    return Ok(false);
                }
            }
        } else if self.last_token_get.is_some() {
            let last_get = self.last_token_get.unwrap();
            if Instant::now().duration_since(last_get) > Duration::from_secs(60000) {
                let potential_csrf_token = self.get_current_token().await;
                match potential_csrf_token {
                    Some(t) => token = t,
                    None => {
                        log_error(
                            "Failed to retrieve xCRSF token, is the roblox API down?".to_string(),
                        )
                        .await;
                        return Ok(false);
                    }
                }
            }
        }

        let client = reqwest::Client::new();
        let response = client
            .patch(format!(
                "https://groups.roblox.com/v1/groups/{}/users/{}",
                group_id, user_id
            ))
            .header("Content-Type", "application/json")
            .header("Refer", "https://www.roblox.com")
            .header("cookie", format!(".ROBLOSECURITY={};", self.cookie))
            .header("X-CSRF-TOKEN", token)
            .json(&SetRankBody {
                role_id: rank.to_role_id(),
            })
            .send()
            .await?;

        if response.status() == 200 {
            Ok(true)
        } else {
            println!("{}", response.text().await.unwrap());
            Ok(false)
        }
    }
}

async fn attempt_login(cookie: &str) -> Result<bool, reqwest::Error> {
    let body = [("session", cookie)];
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.roblox.com/mobileapi/userinfo")
        .form(&body)
        .send()
        .await?;

    if response.status() == 200 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn create_user(cookie: String, should_validate: bool) -> RobloxAccount {
    if !cookie.to_lowercase().contains("warning:-") {
        panic!("Warning: No Roblox warning detected in provided cookie. Ensure you include the entire .ROBLOSECURITY warning.")
    } else {
        if should_validate {
            let logged_in = attempt_login(&cookie).await;
            match logged_in {
                Ok(return_bool) => {
                    if return_bool {
                        return RobloxAccount::new(cookie);
                    } else {
                        panic!("Failed to log in!");
                    }
                }
                Err(e) => panic!("{}", e.to_string()),
            }
        } else {
            return RobloxAccount::new(cookie);
        }
    }
}
