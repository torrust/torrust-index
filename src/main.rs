use std::sync::Arc;
use actix_web::{App, HttpServer, middleware, web};
use actix_cors::Cors;
use std::env;
use torrust::database::Database;
use torrust::{handlers};
use torrust::config::TorrustConfig;
use torrust::common::AppData;
use torrust::auth::AuthorizationService;
use torrust::tracker::TrackerService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cfg = Arc::new(TorrustConfig::new().unwrap());
    let database = Arc::new(Database::new(&cfg.database.connect_url).await);
    let auth = Arc::new(AuthorizationService::new(cfg.clone(), database.clone()));
    let tracker_service = Arc::new(TrackerService::new(cfg.clone(), database.clone()));
    let app_data = Arc::new(
        AppData::new(
            cfg.clone(),
            database.clone(),
            auth.clone(),
            tracker_service.clone()
        )
    );

    // create/update database tables
    sqlx::migrate!().run(&database.pool).await.unwrap();

    // create torrent upload folder
    async_std::fs::create_dir_all(&cfg.storage.upload_path).await?;

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app_data.clone()))
            .wrap(middleware::Logger::default())
            .configure(handlers::init_routes)
    })
        .bind(("0.0.0.0", cfg.net.port))?
        .run()
        .await
}
