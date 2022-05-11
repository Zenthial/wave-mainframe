use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
    fn to_value(&self) -> u32 {
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
}

impl ToString for Ranks {
    fn to_string(&self) -> String {
        match &self {
            Ranks::Chairman => String::from("Chairman"),
            Ranks::Marshal => String::from("Marshal"),
            Ranks::Colonel => String::from("Colonel"),
            Ranks::Captain => String::from("Captain"),
            Ranks::Lieutenant => String::from("Lieutenant"),
            Ranks::Ensign => String::from("Engisn"),
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
