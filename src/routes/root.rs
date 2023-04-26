use actix_web::web;

use crate::routes::about;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/").service(web::resource("").route(web::get().to(about::get))));
}
