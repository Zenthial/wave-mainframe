use actix_web::{
    get,
    web::{Data, ServiceConfig},
    HttpResponse,
};

use crate::AppState;

#[get("leaderboard")]
async fn get_leaderboard(app_state: Data<AppState>) -> HttpResponse {
    let lb = app_state.leaderboard.read();
    if lb.needs_update() {
        drop(lb);
        let mut write_lb = app_state.leaderboard.write();
        write_lb.update(&app_state.database.read()).await;
    }
    let lb = app_state.leaderboard.read();
    let vec = lb.get();

    return HttpResponse::Ok().json(vec);
}

pub fn configure_leaderboard(cfg: &mut ServiceConfig) {
    cfg.service(get_leaderboard);
}
