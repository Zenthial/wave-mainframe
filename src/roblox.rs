#![allow(dead_code)]

use std::collections::HashMap;

use reqwest;
use serde::{Deserialize, Serialize};

use crate::ranks::Ranks;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsernameResponse {
    description: String,
    created: String,
    is_banned: bool,
    external_app_display_name: String,
    pub id: u64,
    pub name: String,
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    id: u64,
    name: String,
    member_count: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleInfo {
    id: u64,
    name: String,
    rank: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserGroupInfo {
    group: GroupInfo,
    role: RoleInfo,
}

#[derive(Serialize, Deserialize)]
pub struct GroupResponse {
    data: Vec<UserGroupInfo>,
}

pub async fn get_user_info_from_id(user_id: u64) -> Result<UsernameResponse, reqwest::Error> {
    let response = reqwest::get(format!("https://users.roblox.com/v1/users/{}", user_id)).await?;
    let username_response = response.json::<UsernameResponse>().await?;

    Ok(username_response)
}

pub async fn get_rank_in_group(group_id: u64, user_id: u64) -> Result<Option<u64>, reqwest::Error> {
    let response = reqwest::get(format!(
        "https://groups.roblox.com/v2/users/{}/groups/roles",
        user_id
    ))
    .await?;

    let group_response = response.json::<GroupResponse>().await?;
    let index = group_response
        .data
        .iter()
        .position(|group_info| group_info.group.id == group_id);

    if let Some(i) = index {
        Ok(Some(group_response.data.get(i).unwrap().role.rank))
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

#[derive(Clone)]
pub struct RobloxAccount {
    cookie: String,
    headers: HashMap<String, String>,
    token: String,
}

impl RobloxAccount {
    fn new(cookie: String) -> RobloxAccount {
        RobloxAccount {
            cookie,
            headers: HashMap::new(),
            token: String::new(),
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

        let response = match response_result {
            Ok(res) => res,
            Err(e) => panic!("{:?}", e),
        };

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
            return Some(token);
        } else {
            return None;
        }
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
        user_id: u64,
        group_id: u64,
        rank: Ranks,
    ) -> Result<bool, reqwest::Error> {
        let mut token = self.token.clone();
        if token == "" {
            let potential_csrf_token = self.get_current_token().await;
            match potential_csrf_token {
                Some(t) => token = t,
                None => panic!("Failed to retrieve xCRSF token, is the roblox API down?"),
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
