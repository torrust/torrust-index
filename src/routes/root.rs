use actix_web::web;

use crate::routes::{about, API_VERSION};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").service(web::resource("").route(web::get().to(about::get))));
    cfg.service(web::scope(&format!("/{API_VERSION}")).service(web::resource("").route(web::get().to(about::get))));
}
