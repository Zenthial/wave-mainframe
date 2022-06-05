use reqwest::Client;
use serde::Serialize;
use std::{fs::File, io::Read};

#[derive(Serialize, Debug)]
struct WebhookBody {
    content: String,
}

pub async fn log_to_discord(message: String) {
    let mut webhook_file = File::open("webhook.txt").expect("file to be able to open");
    let mut webhook = String::new();
    webhook_file.read_to_string(&mut webhook).unwrap();

    let client = Client::new();
    let _response = client
        .post(webhook)
        .json(&WebhookBody { content: message })
        .send()
        .await;
}

pub async fn log_error(message: String) {
    let mut webhook_file = File::open("error-webhook.txt").expect("file to be able to open");
    let mut webhook = String::new();
    webhook_file.read_to_string(&mut webhook).unwrap();

    let client = Client::new();
    let _response = client
        .post(webhook)
        .json(&WebhookBody { content: message })
        .send()
        .await;
}
