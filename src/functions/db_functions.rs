use std::sync::RwLock;

use firebase_realtime_database::{get_oauth_token, Database};

async fn refresh_token(db: &mut Database) {
    let token = get_oauth_token("firebase-key.json").await.unwrap();

    db.set_token(token.as_str().to_string());
}

async fn check_verified(db: &Database) -> bool {
    let response = db.get("/online").await;

    if let Ok(res) = response {
        if res.status() == 401 {
            return false;
        }
    }

    return true;
}

pub async fn safe_to_use(guard: &RwLock<Database>) -> bool {
    let read_guard = guard.read().unwrap();

    if !check_verified(&read_guard).await {
        let mut write_guard = guard.write().unwrap();
        refresh_token(&mut write_guard).await;

        drop(write_guard);
    }

    return true;
}
