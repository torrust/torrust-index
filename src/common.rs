use std::sync::Arc;

use crate::auth::AuthorizationService;
use crate::cache::image::manager::ImageCacheService;
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::mailer;
use crate::tracker::service::Service;
use crate::tracker::statistics_importer::StatisticsImporter;

pub type Username = String;

pub type WebAppData = actix_web::web::Data<Arc<AppData>>;

pub struct AppData {
    pub cfg: Arc<Configuration>,
    pub database: Arc<Box<dyn Database>>,
    pub auth: Arc<AuthorizationService>,
    pub tracker_service: Arc<Service>,
    pub tracker_statistics_importer: Arc<StatisticsImporter>,
    pub mailer: Arc<mailer::Service>,
    pub image_cache_manager: Arc<ImageCacheService>,
}

impl AppData {
    pub fn new(
        cfg: Arc<Configuration>,
        database: Arc<Box<dyn Database>>,
        auth: Arc<AuthorizationService>,
        tracker_service: Arc<Service>,
        tracker_statistics_importer: Arc<StatisticsImporter>,
        mailer: Arc<mailer::Service>,
        image_cache_manager: Arc<ImageCacheService>,
    ) -> AppData {
        AppData {
            cfg,
            database,
            auth,
            tracker_service,
            tracker_statistics_importer,
            mailer,
            image_cache_manager,
        }
    }
}
