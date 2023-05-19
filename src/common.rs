use std::sync::Arc;

use crate::auth::AuthorizationService;
use crate::cache::image::manager::ImageCacheService;
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::services::category::{self, DbCategoryRepository};
use crate::services::torrent::{
    DbTorrentAnnounceUrlRepository, DbTorrentFileRepository, DbTorrentInfoRepository, DbTorrentListingGenerator,
    DbTorrentRepository,
};
use crate::services::user::DbUserRepository;
use crate::services::{proxy, settings, torrent};
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
    // Repositories
    pub category_repository: Arc<DbCategoryRepository>,
    pub user_repository: Arc<DbUserRepository>,
    pub torrent_repository: Arc<DbTorrentRepository>,
    pub torrent_info_repository: Arc<DbTorrentInfoRepository>,
    pub torrent_file_repository: Arc<DbTorrentFileRepository>,
    pub torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
    pub torrent_listing_generator: Arc<DbTorrentListingGenerator>,
    // Services
    pub category_service: Arc<category::Service>,
    pub proxy_service: Arc<proxy::Service>,
    pub settings_service: Arc<settings::Service>,
    pub torrent_service: Arc<torrent::Index>,
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
        // Repositories
        category_repository: Arc<DbCategoryRepository>,
        user_repository: Arc<DbUserRepository>,
        torrent_repository: Arc<DbTorrentRepository>,
        torrent_info_repository: Arc<DbTorrentInfoRepository>,
        torrent_file_repository: Arc<DbTorrentFileRepository>,
        torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
        torrent_listing_generator: Arc<DbTorrentListingGenerator>,
        // Services
        category_service: Arc<category::Service>,
        proxy_service: Arc<proxy::Service>,
        settings_service: Arc<settings::Service>,
        torrent_service: Arc<torrent::Index>,
    ) -> AppData {
        AppData {
            cfg,
            database,
            auth,
            tracker_service,
            tracker_statistics_importer,
            mailer,
            image_cache_manager,
            // Repositories
            category_repository,
            user_repository,
            torrent_repository,
            torrent_info_repository,
            torrent_file_repository,
            torrent_announce_url_repository,
            torrent_listing_generator,
            // Services
            category_service,
            proxy_service,
            settings_service,
            torrent_service,
        }
    }
}
