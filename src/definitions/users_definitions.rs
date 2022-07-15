use super::ranks::{Ranks, STRanks, SableRanks};
use serde::{Deserialize, Serialize};

pub static WIJ_ID: u32 = 3747606;
pub static ST_ID: u32 = 3758883;
pub static SABLE_ID: u32 = 5430057;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Divisions {
    #[serde(default)]
    pub st: Option<STRanks>,
    #[serde(default)]
    pub sable: Option<SableRanks>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    #[serde(default)]
    pub user_id: u32,
    pub name: String,
    #[serde(default)]
    pub points: u32,
    #[serde(default)]
    pub events: u32,
    #[serde(default)]
    pub floor_points: Option<u32>,
    #[serde(default)]
    pub goal_points: Option<u32>,
    #[serde(default)]
    pub rank: Ranks,
    pub divisions: Option<Divisions>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeserializeUser {
    #[serde(default)]
    pub user_id: u32,
    pub name: String,
    #[serde(default)]
    pub points: u32,
    #[serde(default)]
    pub events: u32,
    #[serde(default)]
    pub floor_points: Option<u32>,
    #[serde(default)]
    pub goal_points: Option<u32>,
    #[serde(default)]
    pub rank: String,
    pub divisions: Option<Divisions>,
}
