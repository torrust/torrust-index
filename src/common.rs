use std::sync::Arc;

use crate::auth::AuthorizationService;
use crate::cache::image::manager::ImageCacheService;
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::services::category::{self, DbCategoryRepository};
use crate::services::user::DbUserRepository;
use crate::services::{proxy, settings};
use crate::tracker::statistics_importer::StatisticsImporter;
use crate::{mailer, tracker};
pub type Username = String;

pub type WebAppData = actix_web::web::Data<Arc<AppData>>;

pub struct AppData {
    pub cfg: Arc<Configuration>,
    pub database: Arc<Box<dyn Database>>,
    pub auth: Arc<AuthorizationService>,
    pub tracker_service: Arc<tracker::service::Service>,
    pub tracker_statistics_importer: Arc<StatisticsImporter>,
    pub mailer: Arc<mailer::Service>,
    pub image_cache_manager: Arc<ImageCacheService>,
    pub category_repository: Arc<DbCategoryRepository>,
    pub user_repository: Arc<DbUserRepository>,
    pub category_service: Arc<category::Service>,
    pub proxy_service: Arc<proxy::Service>,
    pub settings_service: Arc<settings::Service>,
}

impl AppData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cfg: Arc<Configuration>,
        database: Arc<Box<dyn Database>>,
        auth: Arc<AuthorizationService>,
        tracker_service: Arc<tracker::service::Service>,
        tracker_statistics_importer: Arc<StatisticsImporter>,
        mailer: Arc<mailer::Service>,
        image_cache_manager: Arc<ImageCacheService>,
        category_repository: Arc<DbCategoryRepository>,
        user_repository: Arc<DbUserRepository>,
        category_service: Arc<category::Service>,
        proxy_service: Arc<proxy::Service>,
        settings_service: Arc<settings::Service>,
    ) -> AppData {
        AppData {
            cfg,
            database,
            auth,
            tracker_service,
            tracker_statistics_importer,
            mailer,
            image_cache_manager,
            category_repository,
            user_repository,
            category_service,
            proxy_service,
            settings_service,
        }
    }
}
