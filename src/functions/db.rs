use parking_lot::RwLock;

use firebase_realtime_database::{get_oauth_token, Database};
use log::info;

async fn refresh_token(db: &mut Database) {
    let token = get_oauth_token("firebase-key.json").await.unwrap();

    db.set_token(token.as_str().to_string());
}

async fn check_verified(db: &Database) -> bool {
    info!("getting");
    let response = db.get("online").await;
    info!("{:?}", response);

    if let Ok(res) = response {
        if res.status() == 401 {
            return false;
        }
    }

    return true;
}

pub async fn safe_to_use(guard: &RwLock<Database>) {
    let read_guard = guard.read();

    if !check_verified(&read_guard).await {
        drop(read_guard);
        info!("getting guard");
        let mut write_guard = guard.write();
        info!("got guard");
        refresh_token(&mut write_guard).await;
    }
}
