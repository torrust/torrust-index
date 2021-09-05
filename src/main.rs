use std::sync::Arc;
use actix_web::{App, HttpServer, middleware, web};
use std::env;
use torrust::data::Database;
use torrust::{handlers};
use torrust::config::TorrustConfig;
use torrust::common::AppData;
use torrust::auth::AuthorizationService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cfg = Arc::new(TorrustConfig::new().unwrap());
    let database = Arc::new(Database::new(&cfg.database.connect_url).await);
    let auth = Arc::new(AuthorizationService::new(cfg.auth.secret_key.clone(), database.clone()));
    let app_data = Arc::new(AppData::new(cfg.clone(), database.clone(), auth.clone()));

    // create/update database tables
    sqlx::migrate!().run(&database.pool).await.unwrap();

    // create torrent upload folder
    async_std::fs::create_dir_all(&cfg.storage.upload_path).await?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .wrap(middleware::Logger::default())
            .configure(handlers::init_routes)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
