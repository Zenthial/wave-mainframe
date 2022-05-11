use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UsernameResponse {
    id: i32,
    username: String,
    avatar_uri: Option<String>,
    avatar_final: bool,
    is_online: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    id: i32,
    name: String,
    member_count: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleInfo {
    id: i32,
    name: String,
    rank: i32,
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

pub async fn get_user_info_from_id(user_id: i32) -> Result<UsernameResponse, reqwest::Error> {
    let response = reqwest::get(format!("https://api.roblox.com/users/{}", user_id)).await?;
    let username_response = response.json::<UsernameResponse>().await?;

    Ok(username_response)
}

pub async fn get_rank_in_group(group_id: i32, user_id: i32) -> Result<i32, reqwest::Error> {
    let response = reqwest::get(format!(
        "https://groups.roblox.com/v2/users/{}/groups/roles",
        user_id
    ))
    .await?;

    let group_response = response.json::<GroupResponse>().await?;
    let index = group_response
        .data
        .iter()
        .position(|user_group_info| user_group_info.group.id == group_id);

    if let Some(i) = index {
        Ok(group_response.data.get(i).unwrap().role.rank)
    } else {
        Ok(-1)
    }
}

pub struct User {
    cookie: String,
}

pub async fn create_user(cookie: String) -> User {
    if !cookie.to_lowercase().contains("warning:-") {
        panic!("Warning: No Roblox warning detected in provided cookie. Ensure you include the entire .ROBLOSECURITY warning.")
    }
}
