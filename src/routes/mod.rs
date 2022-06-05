use actix_web::web::ServiceConfig;

use self::{users::configure_users, verify::configure_verify};

pub mod users;
pub mod verify;

pub fn configure_routes(cfg: &mut ServiceConfig) {
    configure_verify(cfg);
    configure_users(cfg);
}
