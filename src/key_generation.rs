use once_cell::sync::Lazy; // 1.3.1
use rand::Rng;
use std::sync::Mutex;
use std::{fs::File, io::Read};

static KEYS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

static DEFAULT_START_KEYS: u32 = 50;

fn create_key(str_arr: &Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    let mut key = String::new();

    for _ in 0..4 {
        let rand_index = rng.gen_range(0..str_arr.len());
        let random_word = str_arr.get(rand_index).unwrap();
        key.insert_str(0, random_word.as_str());
    }

    key
}

pub fn init_keys() {
    let mut str_buff = String::new();
    let mut words_file = File::open("words.txt").expect("failed to open words file");
    words_file
        .read_to_string(&mut str_buff)
        .expect("to be able to read the file to a string");

    let str_arr: Vec<String> = str_buff.split("\n").map(|s| s.to_string()).collect();
    let mut keys = KEYS.lock().unwrap();

    for _ in 0..DEFAULT_START_KEYS {
        keys.push(create_key(&str_arr));
    }
}

pub fn get_key() -> String {
    let mut keys = KEYS.lock().unwrap();
    if keys.len() == 0 {
        init_keys();
        return get_key();
    } else {
        let return_str = keys.pop().unwrap();
        return return_str;
    }
}
