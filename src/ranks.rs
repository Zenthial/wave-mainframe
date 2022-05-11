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
    SergeantMajorOfTheAlliance,
    StaffSergeant,
    TechSergeant,
    Veteran,
    Corporal,
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
        Ranks::Enlisted
    }
}

impl Ranks {
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
