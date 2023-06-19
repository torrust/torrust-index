use actix_web::web;

pub mod about;
pub mod category;
pub mod proxy;
pub mod root;
pub mod settings;
pub mod tag;
pub mod torrent;
pub mod user;

pub fn init(cfg: &mut web::ServiceConfig) {
    user::init(cfg);
    torrent::init(cfg);
    category::init(cfg);
    settings::init(cfg);
    about::init(cfg);
    proxy::init(cfg);
    tag::init(cfg);
    root::init(cfg);
}
