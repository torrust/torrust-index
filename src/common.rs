use std::sync::Arc;

use crate::cache::image::manager::ImageCacheService;
use crate::config::Configuration;
use crate::databases::database::Database;
use crate::services::authentication::{DbUserAuthenticationRepository, JsonWebToken, Service};
use crate::services::category::{self, DbCategoryRepository};
use crate::services::tag::{self, DbTagRepository};
use crate::services::torrent::{
    DbCanonicalInfoHashGroupRepository, DbTorrentAnnounceUrlRepository, DbTorrentFileRepository, DbTorrentInfoRepository,
    DbTorrentListingGenerator, DbTorrentRepository, DbTorrentTagRepository,
};
use crate::services::user::{self, DbBannedUserList, DbUserProfileRepository, Repository};
use crate::services::{about, proxy, settings, torrent};
use crate::tracker::statistics_importer::StatisticsImporter;
use crate::web::api::server::v1::auth::Authentication;
use crate::{mailer, tracker};

pub type Username = String;

pub struct AppData {
    pub cfg: Arc<Configuration>,
    pub database: Arc<Box<dyn Database>>,
    pub json_web_token: Arc<JsonWebToken>,
    pub auth: Arc<Authentication>,
    pub authentication_service: Arc<Service>,
    pub tracker_service: Arc<tracker::service::Service>,
    pub tracker_statistics_importer: Arc<StatisticsImporter>,
    pub mailer: Arc<mailer::Service>,
    pub image_cache_manager: Arc<ImageCacheService>,
    // Repositories
    pub category_repository: Arc<DbCategoryRepository>,
    pub tag_repository: Arc<DbTagRepository>,
    pub user_repository: Arc<Box<dyn Repository>>,
    pub user_authentication_repository: Arc<DbUserAuthenticationRepository>,
    pub user_profile_repository: Arc<DbUserProfileRepository>,
    pub torrent_repository: Arc<DbTorrentRepository>,
    pub torrent_info_hash_repository: Arc<DbCanonicalInfoHashGroupRepository>,
    pub torrent_info_repository: Arc<DbTorrentInfoRepository>,
    pub torrent_file_repository: Arc<DbTorrentFileRepository>,
    pub torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
    pub torrent_tag_repository: Arc<DbTorrentTagRepository>,
    pub torrent_listing_generator: Arc<DbTorrentListingGenerator>,
    pub banned_user_list: Arc<DbBannedUserList>,
    // Services
    pub category_service: Arc<category::Service>,
    pub tag_service: Arc<tag::Service>,
    pub proxy_service: Arc<proxy::Service>,
    pub settings_service: Arc<settings::Service>,
    pub torrent_service: Arc<torrent::Index>,
    pub registration_service: Arc<user::RegistrationService>,
    pub profile_service: Arc<user::ProfileService>,
    pub ban_service: Arc<user::BanService>,
    pub about_service: Arc<about::Service>,
}

impl AppData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cfg: Arc<Configuration>,
        database: Arc<Box<dyn Database>>,
        json_web_token: Arc<JsonWebToken>,
        auth: Arc<Authentication>,
        authentication_service: Arc<Service>,
        tracker_service: Arc<tracker::service::Service>,
        tracker_statistics_importer: Arc<StatisticsImporter>,
        mailer: Arc<mailer::Service>,
        image_cache_manager: Arc<ImageCacheService>,
        // Repositories
        category_repository: Arc<DbCategoryRepository>,
        tag_repository: Arc<DbTagRepository>,
        user_repository: Arc<Box<dyn Repository>>,
        user_authentication_repository: Arc<DbUserAuthenticationRepository>,
        user_profile_repository: Arc<DbUserProfileRepository>,
        torrent_repository: Arc<DbTorrentRepository>,
        torrent_info_hash_repository: Arc<DbCanonicalInfoHashGroupRepository>,
        torrent_info_repository: Arc<DbTorrentInfoRepository>,
        torrent_file_repository: Arc<DbTorrentFileRepository>,
        torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
        torrent_tag_repository: Arc<DbTorrentTagRepository>,
        torrent_listing_generator: Arc<DbTorrentListingGenerator>,
        banned_user_list: Arc<DbBannedUserList>,
        // Services
        category_service: Arc<category::Service>,
        tag_service: Arc<tag::Service>,
        proxy_service: Arc<proxy::Service>,
        settings_service: Arc<settings::Service>,
        torrent_service: Arc<torrent::Index>,
        registration_service: Arc<user::RegistrationService>,
        profile_service: Arc<user::ProfileService>,
        ban_service: Arc<user::BanService>,
        about_service: Arc<about::Service>,
    ) -> AppData {
        AppData {
            cfg,
            database,
            json_web_token,
            auth,
            authentication_service,
            tracker_service,
            tracker_statistics_importer,
            mailer,
            image_cache_manager,
            // Repositories
            category_repository,
            tag_repository,
            user_repository,
            user_authentication_repository,
            user_profile_repository,
            torrent_repository,
            torrent_info_hash_repository,
            torrent_info_repository,
            torrent_file_repository,
            torrent_announce_url_repository,
            torrent_tag_repository,
            torrent_listing_generator,
            banned_user_list,
            // Services
            category_service,
            tag_service,
            proxy_service,
            settings_service,
            torrent_service,
            registration_service,
            profile_service,
            ban_service,
            about_service,
        }
    }
}
