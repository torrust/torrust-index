use actix_web::web;

pub mod about;
pub mod category;
pub mod root;
pub mod settings;
pub mod torrent;
pub mod user;
pub mod proxy;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    user::init_routes(cfg);
    torrent::init_routes(cfg);
    category::init_routes(cfg);
    settings::init_routes(cfg);
    about::init_routes(cfg);
    proxy::init_routes(cfg);
    root::init_routes(cfg);
}
