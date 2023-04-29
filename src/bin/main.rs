use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use torrust_index_backend::auth::AuthorizationService;
use torrust_index_backend::bootstrap::logging;
use torrust_index_backend::common::AppData;
use torrust_index_backend::config::{Configuration, CONFIG_ENV_VAR_NAME, CONFIG_PATH};
use torrust_index_backend::databases::database::connect_database;
use torrust_index_backend::mailer::MailerService;
use torrust_index_backend::routes;
use torrust_index_backend::tracker::TrackerService;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = init_configuration().await;

    logging::setup();

    let cfg = Arc::new(configuration);

    let settings = cfg.settings.read().await;

    let database = Arc::new(
        connect_database(&settings.database.connect_url)
            .await
            .expect("Database error."),
    );

    let auth = Arc::new(AuthorizationService::new(cfg.clone(), database.clone()));
    let tracker_service = Arc::new(TrackerService::new(cfg.clone(), database.clone()));
    let mailer_service = Arc::new(MailerService::new(cfg.clone()).await);
    let app_data = Arc::new(AppData::new(
        cfg.clone(),
        database.clone(),
        auth.clone(),
        tracker_service.clone(),
        mailer_service.clone(),
    ));

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

    // todo: get IP from settings
    let ip = "0.0.0.0".to_string();
    let port = settings.net.port;

    drop(settings);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app_data.clone()))
            .wrap(middleware::Logger::default())
            .configure(routes::init_routes)
    })
    .bind((ip.clone(), port))
    .expect("can't bind server to socket address");

    let server_port = server.addrs()[0].port();

    println!("Listening on http://{}:{}", ip, server_port);

    server.run().await
}

async fn init_configuration() -> Configuration {
    if env::var(CONFIG_ENV_VAR_NAME).is_ok() {
        println!("Loading configuration from env var `{}`", CONFIG_ENV_VAR_NAME);

        Configuration::load_from_env_var(CONFIG_ENV_VAR_NAME).unwrap()
    } else {
        println!("Loading configuration from config file `{}`", CONFIG_PATH);

        match Configuration::load_from_file().await {
            Ok(config) => config,
            Err(error) => {
                panic!("{}", error)
            }
        }
    }
}
