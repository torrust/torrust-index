use std::net::SocketAddr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::{middleware, web, App, HttpServer};
use log::info;

use crate::auth::AuthorizationService;
use crate::bootstrap::logging;
use crate::cache::image::manager::ImageCacheService;
use crate::common::AppData;
use crate::config::Configuration;
use crate::databases::database;
use crate::tracker::statistics_importer::StatisticsImporter;
use crate::{mailer, routes, tracker};

pub struct Running {
    pub api_server: Server,
    pub socket_address: SocketAddr,
    pub tracker_data_importer_handle: tokio::task::JoinHandle<()>,
}

pub async fn run(configuration: Configuration) -> Running {
    logging::setup();

    let cfg = Arc::new(configuration);

    // Get configuration settings needed to build the app dependencies and
    // services: main API server and tracker torrents importer.

    let settings = cfg.settings.read().await;

    let database_connect_url = settings.database.connect_url.clone();
    let database_torrent_info_update_interval = settings.database.torrent_info_update_interval;
    let net_port = settings.net.port;

    // IMPORTANT: drop settings before starting server to avoid read locks that
    // leads to requests hanging.
    drop(settings);

    // Build app dependencies

    let database = Arc::new(database::connect(&database_connect_url).await.expect("Database error."));
    let auth = Arc::new(AuthorizationService::new(cfg.clone(), database.clone()));
    let tracker_service = Arc::new(tracker::service::Service::new(cfg.clone(), database.clone()).await);
    let tracker_statistics_importer =
        Arc::new(StatisticsImporter::new(cfg.clone(), tracker_service.clone(), database.clone()).await);
    let mailer_service = Arc::new(mailer::Service::new(cfg.clone()).await);
    let image_cache_service = Arc::new(ImageCacheService::new(cfg.clone()).await);

    // Build app container

    let app_data = Arc::new(AppData::new(
        cfg.clone(),
        database.clone(),
        auth.clone(),
        tracker_service.clone(),
        tracker_statistics_importer.clone(),
        mailer_service,
        image_cache_service,
    ));

    // Start repeating task to import tracker torrent data and updating
    // seeders and leechers info.

    let weak_tracker_statistics_importer = Arc::downgrade(&tracker_statistics_importer);

    let tracker_statistics_importer_handle = tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(database_torrent_info_update_interval);
        let mut interval = tokio::time::interval(interval);
        interval.tick().await; // first tick is immediate...
        loop {
            interval.tick().await;
            if let Some(tracker) = weak_tracker_statistics_importer.upgrade() {
                let _ = tracker.import_all_torrents_statistics().await;
            } else {
                break;
            }
        }
    });

    // Start main API server

    // todo: get IP from settings
    let ip = "0.0.0.0".to_string();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(app_data.clone()))
            .wrap(middleware::Logger::default())
            .configure(routes::init)
    })
    .bind((ip, net_port))
    .expect("can't bind server to socket address");

    let socket_address = server.addrs()[0];

    let running_server = server.run();

    let starting_message = format!("Listening on http://{socket_address}");
    info!("{}", starting_message);
    // Logging could be disabled or redirected to file. So print to stdout too.
    println!("{starting_message}");

    Running {
        api_server: running_server,
        socket_address,
        tracker_data_importer_handle: tracker_statistics_importer_handle,
    }
}
