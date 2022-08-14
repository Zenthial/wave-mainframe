use super::ranks::{Ranks, STRanks, SableRanks};
use chrono::Utc;
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
pub struct BPLog {
    time: String,
    awarder: u32,
    amount: i32,
    place_name: Option<String>,
}

impl BPLog {
    pub fn new(awarder: u32, amount: i32) -> Self {
        BPLog {
            time: Utc::now().to_string(),
            awarder,
            amount,
            place_name: None,
        }
    }

    pub fn add_place(&mut self, place_name: &str) {
        self.place_name = Some(place_name.to_string());
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    #[serde(default)]
    pub user_id: u32,
    pub name: String,
    #[serde(default)]
    pub points: i32,
    #[serde(default)]
    pub total_points: i32,
    #[serde(default)]
    pub events: u32,
    #[serde(default)]
    pub floor_points: Option<i32>,
    #[serde(default)]
    pub goal_points: Option<i32>,
    #[serde(default)]
    pub rank: Ranks,
    pub divisions: Option<Divisions>,

    #[serde(default)]
    pub prestige: Option<i32>,

    pub bp_logs: Option<Vec<BPLog>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeserializeUser {
    #[serde(default)]
    pub user_id: u32,
    pub name: String,
    #[serde(default)]
    pub points: i32,
    #[serde(default)]
    pub total_points: i32,
    #[serde(default)]
    pub events: u32,
    #[serde(default)]
    pub floor_points: Option<i32>,
    #[serde(default)]
    pub goal_points: Option<i32>,
    #[serde(default)]
    pub rank: String,
    pub divisions: Option<Divisions>,

    #[serde(default)]
    pub prestige: Option<i32>,

    pub bp_logs: Option<Vec<BPLog>>,
}
