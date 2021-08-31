use std::sync::Arc;

use actix_web::{App, HttpServer, middleware};
use lazy_static::lazy_static;

pub use crate::config::TorrustConfig;
pub use crate::data::Data;

mod handlers;
mod data;
mod config;
mod errors;
mod models;

pub type AppData = actix_web::web::Data<Arc<crate::data::Data>>;

lazy_static! {
    pub static ref CONFIG: TorrustConfig = TorrustConfig::new().unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = Data::new().await;
    sqlx::migrate!().run(&data.db).await.unwrap();
    let data = actix_web::web::Data::new(data);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .configure(handlers::init_routes)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
