use super::ranks::{Ranks, STRanks, SableRanks};
use serde::{Deserialize, Serialize};

pub static WIJ_ID: u64 = 3747606;
pub static ST_ID: u64 = 3758883;
pub static SABLE_ID: u64 = 5430057;

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
    pub user_id: u64,
    pub name: String,
    #[serde(default)]
    pub points: u64,
    #[serde(default)]
    pub events: u64,
    #[serde(default)]
    pub floor_points: Option<u64>,
    #[serde(default)]
    pub goal_points: Option<u64>,
    #[serde(default)]
    pub rank: Ranks,
    pub divisions: Option<Divisions>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeserializeUser {
    #[serde(default)]
    pub user_id: u64,
    pub name: String,
    #[serde(default)]
    pub points: u64,
    #[serde(default)]
    pub events: u64,
    #[serde(default)]
    pub floor_points: Option<u64>,
    #[serde(default)]
    pub goal_points: Option<u64>,
    #[serde(default)]
    pub rank: String,
    pub divisions: Option<Divisions>,
}
