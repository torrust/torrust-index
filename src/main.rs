use std::sync::Arc;
use actix_web::{App, HttpServer, middleware, web};
use actix_cors::Cors;
use torrust_index_backend::{routes};
use torrust_index_backend::config::{Configuration};
use torrust_index_backend::common::AppData;
use torrust_index_backend::auth::AuthorizationService;
use torrust_index_backend::databases::database::connect_database;
use torrust_index_backend::tracker::TrackerService;
use torrust_index_backend::mailer::MailerService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cfg = match Configuration::load_from_file().await {
        Ok(config) => Arc::new(config),
        Err(error) => {
            panic!("{}", error)
        }
    };

    let settings = cfg.settings.read().await;

    let database = Arc::new(connect_database(&settings.database.db_driver, &settings.database.connect_url).await);
    let auth = Arc::new(AuthorizationService::new(cfg.clone(), database.clone()));
    let tracker_service = Arc::new(TrackerService::new(cfg.clone(), database.clone()));
    let mailer_service = Arc::new(MailerService::new(cfg.clone()).await);
    let app_data = Arc::new(
        AppData::new(
            cfg.clone(),
            database.clone(),
            auth.clone(),
            tracker_service.clone(),
            mailer_service.clone(),
        )
    );

    let interval = settings.database.torrent_info_update_interval;
    let weak_tracker_service = Arc::downgrade(&tracker_service);

    // repeating task, update all seeders and leechers info
    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(interval);
        let mut interval = tokio::time::interval(interval);
        interval.tick().await; // first tick is immediate...
        loop {
            interval.tick().await;
            if let Some(tracker) = weak_tracker_service.upgrade() {
                let _ = tracker.update_torrents().await;
            } else {
                break;
            }
        }
    });

    let port = settings.net.port;

    drop(settings);

    println!("Listening on 0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app_data.clone()))
            .wrap(middleware::Logger::default())
            .configure(routes::init_routes)
    })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
