#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Ranks {
    Chairman,
    Marshal,
    Colonel,
    Captain,
    Lieutenant,
    Ensign,
    #[serde(rename = "Sergeant Major of the Alliance")]
    SergeantMajorOfTheAlliance,
    #[serde(rename = "Staff Sergeant")]
    StaffSergeant,
    #[serde(rename = "Tech Sergeant")]
    TechSergeant,
    Veteran,
    Corporal,
    #[serde(rename = "Lance Corporal")]
    LanceCorporal,
    Sentinel,
    Fleetman,
    Specialist,
    Operative,
    Trooper,
    Enlisted,
}

impl Default for Ranks {
    fn default() -> Self {
        Self::Enlisted
    }
}

impl Ranks {
    pub fn from_value(value: u64) -> Option<Ranks> {
        match value {
            255 => Some(Ranks::Chairman),
            254 => Some(Ranks::Marshal),
            205 => Some(Ranks::Colonel),
            198 => Some(Ranks::Captain),
            196 => Some(Ranks::Lieutenant),
            193 => Some(Ranks::Ensign),
            192 => Some(Ranks::SergeantMajorOfTheAlliance),
            191 => Some(Ranks::StaffSergeant),
            190 => Some(Ranks::TechSergeant),
            26 => Some(Ranks::Veteran),
            25 => Some(Ranks::Corporal),
            21 => Some(Ranks::LanceCorporal),
            19 => Some(Ranks::Sentinel),
            17 => Some(Ranks::Fleetman),
            15 => Some(Ranks::Specialist),
            9 => Some(Ranks::Operative),
            7 => Some(Ranks::Trooper),
            5 => Some(Ranks::Enlisted),
            _ => None,
        }
    }

    pub fn to_value(&self) -> u32 {
        match &self {
            Ranks::Chairman => 255,
            Ranks::Marshal => 254,
            Ranks::Colonel => 205,
            Ranks::Captain => 198,
            Ranks::Lieutenant => 196,
            Ranks::Ensign => 193,
            Ranks::SergeantMajorOfTheAlliance => 192,
            Ranks::StaffSergeant => 191,
            Ranks::TechSergeant => 190,
            Ranks::Veteran => 26,
            Ranks::Corporal => 25,
            Ranks::LanceCorporal => 21,
            Ranks::Sentinel => 19,
            Ranks::Fleetman => 17,
            Ranks::Specialist => 15,
            Ranks::Operative => 9,
            Ranks::Trooper => 7,
            Ranks::Enlisted => 5,
        }
    }

    pub fn to_role_id(&self) -> u32 {
        match &self {
            Ranks::Chairman => 25617739,
            Ranks::Marshal => 25617740,
            Ranks::Colonel => 25617767,
            Ranks::Captain => 25617779,
            Ranks::Lieutenant => 25617781,
            Ranks::Ensign => 25617796,
            Ranks::SergeantMajorOfTheAlliance => 80131938,
            Ranks::StaffSergeant => 80131906,
            Ranks::TechSergeant => 80131913,
            Ranks::Veteran => 26253933,
            Ranks::Corporal => 25617802,
            Ranks::LanceCorporal => 26539946,
            Ranks::Sentinel => 26539927,
            Ranks::Fleetman => 26539923,
            Ranks::Specialist => 25617809,
            Ranks::Operative => 26539897,
            Ranks::Trooper => 26539881,
            Ranks::Enlisted => 25617741,
        }
    }

    pub fn get_next(&self) -> Option<Ranks> {
        match &self {
            Ranks::Chairman => None,
            Ranks::Marshal => Some(Ranks::Chairman),
            Ranks::Colonel => Some(Ranks::Marshal),
            Ranks::Captain => Some(Ranks::Colonel),
            Ranks::Lieutenant => Some(Ranks::Captain),
            Ranks::Ensign => Some(Ranks::Lieutenant),
            Ranks::SergeantMajorOfTheAlliance => Some(Ranks::Ensign),
            Ranks::StaffSergeant => Some(Ranks::SergeantMajorOfTheAlliance),
            Ranks::TechSergeant => Some(Ranks::TechSergeant),
            Ranks::Veteran => None,
            Ranks::Corporal => Some(Ranks::TechSergeant),
            Ranks::LanceCorporal => Some(Ranks::Corporal),
            Ranks::Sentinel => Some(Ranks::LanceCorporal),
            Ranks::Fleetman => Some(Ranks::Sentinel),
            Ranks::Specialist => Some(Ranks::Fleetman),
            Ranks::Operative => Some(Ranks::Specialist),
            Ranks::Trooper => Some(Ranks::Operative),
            Ranks::Enlisted => Some(Ranks::Trooper),
        }
    }

    pub fn get_prev(&self) -> Option<Ranks> {
        match &self {
            Ranks::Chairman => Some(Ranks::Marshal),
            Ranks::Marshal => Some(Ranks::Colonel),
            Ranks::Colonel => Some(Ranks::Captain),
            Ranks::Captain => Some(Ranks::Lieutenant),
            Ranks::Lieutenant => Some(Ranks::Ensign),
            Ranks::Ensign => Some(Ranks::SergeantMajorOfTheAlliance),
            Ranks::SergeantMajorOfTheAlliance => Some(Ranks::StaffSergeant),
            Ranks::StaffSergeant => Some(Ranks::TechSergeant),
            Ranks::TechSergeant => Some(Ranks::Corporal),
            Ranks::Veteran => None,
            Ranks::Corporal => Some(Ranks::LanceCorporal),
            Ranks::LanceCorporal => Some(Ranks::Sentinel),
            Ranks::Sentinel => Some(Ranks::Fleetman),
            Ranks::Fleetman => Some(Ranks::Specialist),
            Ranks::Specialist => Some(Ranks::Operative),
            Ranks::Operative => Some(Ranks::Trooper),
            Ranks::Trooper => Some(Ranks::Enlisted),
            Ranks::Enlisted => None,
        }
    }

    pub fn inverse_to_string(str: String) -> Option<Ranks> {
        match str.as_ref() {
            "Chairman" => Some(Ranks::Chairman),
            "Marshal" => Some(Ranks::Marshal),
            "Colonel" => Some(Ranks::Colonel),
            "Captain" => Some(Ranks::Captain),
            "Lieutenant" => Some(Ranks::Lieutenant),
            "Ensign" => Some(Ranks::Ensign),
            "Sergeant Major of the Alliance" => Some(Ranks::SergeantMajorOfTheAlliance),
            "Staff Sergeant" => Some(Ranks::StaffSergeant),
            "Tech Sergeant" => Some(Ranks::TechSergeant),
            "Veteran" => Some(Ranks::Veteran),
            "Corporal" => Some(Ranks::Corporal),
            "Lance Corporal" => Some(Ranks::LanceCorporal),
            "Sentinel" => Some(Ranks::Sentinel),
            "Fleetman" => Some(Ranks::Fleetman),
            "Specialist" => Some(Ranks::Specialist),
            "Operative" => Some(Ranks::Operative),
            "Trooper" => Some(Ranks::Trooper),
            "Enlisted" => Some(Ranks::Enlisted),
            _ => None,
        }
    }
}

impl ToString for Ranks {
    fn to_string(&self) -> String {
        match &self {
            Ranks::Chairman => String::from("Chairman"),
            Ranks::Marshal => String::from("Marshal"),
            Ranks::Colonel => String::from("Colonel"),
            Ranks::Captain => String::from("Captain"),
            Ranks::Lieutenant => String::from("Lieutenant"),
            Ranks::Ensign => String::from("Ensign"),
            Ranks::SergeantMajorOfTheAlliance => String::from("Sergeant Major of the Alliance"),
            Ranks::StaffSergeant => String::from("Staff Sergeant"),
            Ranks::TechSergeant => String::from("Tech Sergeant"),
            Ranks::Veteran => String::from("Veteran"),
            Ranks::Corporal => String::from("Corporal"),
            Ranks::LanceCorporal => String::from("Lance Corporal"),
            Ranks::Sentinel => String::from("Sentinel"),
            Ranks::Fleetman => String::from("Fleetman"),
            Ranks::Specialist => String::from("Specialist"),
            Ranks::Operative => String::from("Operative"),
            Ranks::Trooper => String::from("Trooper"),
            Ranks::Enlisted => String::from("Enlisted"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum STRanks {
    Chairman,
    Marshal,
    #[serde(rename = "Chief of Staff")]
    ChiefOfStaff,
    #[serde(rename = "Chief Advisor")]
    ChiefAdvisor,
    #[serde(rename = "Ops Chief")]
    OpsChief,
    Infiltrator,
    Operative,
    Trooper,
    Veteran,
}

impl Default for STRanks {
    fn default() -> Self {
        Self::Trooper
    }
}

impl ToString for STRanks {
    fn to_string(&self) -> String {
        match &self {
            STRanks::Chairman => String::from("Chairman"),
            STRanks::Marshal => String::from("Marshal"),
            STRanks::ChiefOfStaff => String::from("Chief of Staff"),
            STRanks::ChiefAdvisor => String::from("Chief Advisor"),
            STRanks::OpsChief => String::from("Ops Chief"),
            STRanks::Infiltrator => String::from("Infiltrator"),
            STRanks::Operative => String::from("Operative"),
            STRanks::Trooper => String::from("Trooper"),
            STRanks::Veteran => String::from("Veteran"),
        }
    }
}

impl STRanks {
    pub fn from_value(value: u64) -> Option<Self> {
        match value {
            255 => Some(Self::Chairman),
            245 => Some(Self::Marshal),
            235 => Some(Self::ChiefOfStaff),
            225 => Some(Self::ChiefAdvisor),
            220 => Some(Self::OpsChief),
            135 => Some(Self::Infiltrator),
            100 => Some(Self::Operative),
            97 => Some(Self::Trooper),
            96 => Some(Self::Veteran),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SableRanks {
    Chairman,
    Marshal,
    Executive,
    Consultant,
    Contractor,
}

impl Default for SableRanks {
    fn default() -> Self {
        Self::Contractor
    }
}

impl ToString for SableRanks {
    fn to_string(&self) -> String {
        match &self {
            SableRanks::Chairman => String::from("Chairman"),
            SableRanks::Marshal => String::from("Marshal"),
            SableRanks::Executive => String::from("Executive"),
            SableRanks::Consultant => String::from("Consultant"),
            SableRanks::Contractor => String::from("Contractor"),
        }
    }
}

impl SableRanks {
    pub fn from_value(value: u64) -> Option<Self> {
        match value {
            255 => Some(SableRanks::Chairman),
            254 => Some(SableRanks::Marshal),
            250 => Some(SableRanks::Executive),
            200 => Some(SableRanks::Consultant),
            100 => Some(SableRanks::Contractor),
            _ => None,
        }
    }
}
