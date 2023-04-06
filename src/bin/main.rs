use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use torrust_index_backend::auth::AuthorizationService;
use torrust_index_backend::bootstrap::logging;
use torrust_index_backend::cache::cache::BytesCache;
use torrust_index_backend::cache::image::manager::{ImageCacheManager, ImageCacheManagerConfig};
use torrust_index_backend::common::AppData;
use torrust_index_backend::config::{Configuration, CONFIG_ENV_VAR_NAME, CONFIG_PATH};
use torrust_index_backend::databases::database::connect_database;
use torrust_index_backend::mailer::MailerService;
use torrust_index_backend::routes;
use torrust_index_backend::tracker::TrackerService;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

    let max_image_request_timeout_ms = cfg.settings.read().await.cache.image_cache_max_request_timeout_ms;
    let max_image_size = cfg.settings.read().await.cache.image_cache_entry_size_limit;
    let user_quota_period_seconds = cfg.settings.read().await.cache.image_cache_user_quota_period_seconds;
    let user_quota_bytes = cfg.settings.read().await.cache.image_cache_user_quota_bytes;
    let image_cache_capacity = cfg.settings.read().await.cache.image_cache_capacity;

    let image_cache_manager_config = ImageCacheManagerConfig {
        max_image_request_timeout_ms,
        max_image_size,
        user_quota_period_seconds,
        user_quota_bytes,
    };

    let image_cache =
        BytesCache::with_capacity_and_entry_size_limit(image_cache_capacity, image_cache_manager_config.max_image_size)
            .expect("Could not create image cache.");

    let image_cache_manager = Arc::new(ImageCacheManager::new(image_cache, image_cache_manager_config));

    let app_data = Arc::new(AppData::new(
        cfg.clone(),
        database.clone(),
        auth.clone(),
        tracker_service.clone(),
        mailer_service.clone(),
        image_cache_manager,
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

    let port = settings.net.port;

    drop(settings);

    println!("Listening on http://0.0.0.0:{}", port);

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
