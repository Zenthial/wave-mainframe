use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
};

use firebase_realtime_database::Database;
use log::info;
use serde_json::{from_str, to_string};
use std::io;

use crate::definitions::users::User;

pub async fn write_users(db: &Database) -> io::Result<()> {
    let response_result = db.get("users").await;

    if response_result.is_err() {
        info!(
            "lb refresh errored with error {}",
            response_result.unwrap_err().message
        );
        return Ok(());
    }

    let map_result = response_result
        .unwrap()
        .json::<HashMap<String, User>>()
        .await;
    if map_result.is_err() {
        info!("lb jsonify failed with error {:?}", map_result.unwrap_err());
        return Ok(());
    }

    let mut vec: Vec<User> = map_result.unwrap().values().map(|v| v.to_owned()).collect();
    vec.sort_by(|a, b| b.points.cmp(&a.points));

    let mut file = File::create("users")?;
    file.write(to_string(&vec)?.as_bytes())?;

    Ok(())
}

pub fn read_users() -> io::Result<Vec<User>> {
    let users_file = File::open("users")?;
    let reader = BufReader::new(users_file);
    let users_string: String = reader
        .lines()
        .map(|s| {
            if s.is_ok() {
                s.unwrap()
            } else {
                "".to_string()
            }
        })
        .collect();

    let users_vec: Vec<User> = from_str(users_string.as_str())?;
    Ok(users_vec)
}
