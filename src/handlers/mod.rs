use actix_web::web;

pub mod user;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    user::init_routes(cfg);
}
