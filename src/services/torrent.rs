use std::sync::Arc;

use serde_derive::Deserialize;

use super::category::DbCategoryRepository;
use super::user::DbUserRepository;
use crate::config::Configuration;
use crate::databases::database::{Category, Database, Error, Sorting};
use crate::errors::ServiceError;
use crate::models::info_hash::InfoHash;
use crate::models::response::{DeletedTorrentResponse, TorrentResponse, TorrentsResponse};
use crate::models::torrent::{TorrentId, TorrentListing, TorrentRequest};
use crate::models::torrent_file::{DbTorrentInfo, Torrent, TorrentFile};
use crate::models::user::UserId;
use crate::tracker::statistics_importer::StatisticsImporter;
use crate::{tracker, AsCSV};

pub struct Index {
    configuration: Arc<Configuration>,
    tracker_statistics_importer: Arc<StatisticsImporter>,
    tracker_service: Arc<tracker::service::Service>,
    user_repository: Arc<DbUserRepository>,
    category_repository: Arc<DbCategoryRepository>,
    torrent_repository: Arc<DbTorrentRepository>,
    torrent_info_repository: Arc<DbTorrentInfoRepository>,
    torrent_file_repository: Arc<DbTorrentFileRepository>,
    torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
    torrent_listing_generator: Arc<DbTorrentListingGenerator>,
}

/// User request to generate a torrent listing.
#[derive(Debug, Deserialize)]
pub struct ListingRequest {
    pub page_size: Option<u8>,
    pub page: Option<u32>,
    pub sort: Option<Sorting>,
    /// Expects comma separated string, eg: "?categories=movie,other,app"
    pub categories: Option<String>,
    pub search: Option<String>,
}

/// Internal specification for torrent listings.
#[derive(Debug, Deserialize)]
pub struct ListingSpecification {
    pub search: Option<String>,
    pub categories: Option<Vec<String>>,
    pub sort: Sorting,
    pub offset: u64,
    pub page_size: u8,
}

impl Index {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        configuration: Arc<Configuration>,
        tracker_statistics_importer: Arc<StatisticsImporter>,
        tracker_service: Arc<tracker::service::Service>,
        user_repository: Arc<DbUserRepository>,
        category_repository: Arc<DbCategoryRepository>,
        torrent_repository: Arc<DbTorrentRepository>,
        torrent_info_repository: Arc<DbTorrentInfoRepository>,
        torrent_file_repository: Arc<DbTorrentFileRepository>,
        torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
        torrent_listing_repository: Arc<DbTorrentListingGenerator>,
    ) -> Self {
        Self {
            configuration,
            tracker_statistics_importer,
            tracker_service,
            user_repository,
            category_repository,
            torrent_repository,
            torrent_info_repository,
            torrent_file_repository,
            torrent_announce_url_repository,
            torrent_listing_generator: torrent_listing_repository,
        }
    }

    /// Adds a torrent to the index.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * Unable to get the user from the database.
    /// * Unable to get torrent request from payload.
    /// * Unable to get the category from the database.
    /// * Unable to insert the torrent into the database.
    /// * Unable to add the torrent to the whitelist.
    pub async fn add_torrent(&self, mut torrent_request: TorrentRequest, user_id: UserId) -> Result<TorrentId, ServiceError> {
        torrent_request.torrent.set_announce_urls(&self.configuration).await;

        let category = self
            .category_repository
            .get_by_name(&torrent_request.fields.category)
            .await
            .map_err(|_| ServiceError::InvalidCategory)?;

        let torrent_id = self.torrent_repository.add(&torrent_request, user_id, category).await?;

        let _ = self
            .tracker_statistics_importer
            .import_torrent_statistics(torrent_id, &torrent_request.torrent.info_hash())
            .await;

        // We always whitelist the torrent on the tracker because even if the tracker mode is `public`
        // it could be changed to `private` later on.
        if let Err(e) = self
            .tracker_service
            .whitelist_info_hash(torrent_request.torrent.info_hash())
            .await
        {
            // If the torrent can't be whitelisted somehow, remove the torrent from database
            let _ = self.torrent_repository.delete(&torrent_id).await;
            return Err(e);
        }

        Ok(torrent_id)
    }

    /// Gets a torrent from the Index.
    ///
    /// # Errors
    ///
    /// This function will return an error if unable to get the torrent from the
    /// database.
    pub async fn get_torrent(&self, info_hash: &InfoHash, opt_user_id: Option<UserId>) -> Result<Torrent, ServiceError> {
        let mut torrent = self.torrent_repository.get_by_info_hash(info_hash).await?;

        let tracker_url = self.get_tracker_url().await;

        // Add personal tracker url or default tracker url
        match opt_user_id {
            Some(user_id) => {
                let personal_announce_url = self
                    .tracker_service
                    .get_personal_announce_url(user_id)
                    .await
                    .unwrap_or(tracker_url);
                torrent.announce = Some(personal_announce_url.clone());
                if let Some(list) = &mut torrent.announce_list {
                    let vec = vec![personal_announce_url];
                    list.insert(0, vec);
                }
            }
            None => {
                torrent.announce = Some(tracker_url);
            }
        }

        Ok(torrent)
    }

    /// Delete a Torrent from the Index
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * Unable to get the user who is deleting the torrent (logged-in user).
    /// * The user does not have permission to delete the torrent.
    /// * Unable to get the torrent listing from it's ID.
    /// * Unable to delete the torrent from the database.
    pub async fn delete_torrent(&self, info_hash: &InfoHash, user_id: &UserId) -> Result<DeletedTorrentResponse, ServiceError> {
        let user = self.user_repository.get_compact_user(user_id).await?;

        // Only administrator can delete torrents.
        // todo: move this to an authorization service.
        if !user.administrator {
            return Err(ServiceError::Unauthorized);
        }

        let torrent_listing = self.torrent_listing_generator.one_torrent_by_info_hash(info_hash).await?;

        self.torrent_repository.delete(&torrent_listing.torrent_id).await?;

        // Remove info-hash from tracker whitelist
        let _ = self
            .tracker_service
            .remove_info_hash_from_whitelist(info_hash.to_string())
            .await;

        Ok(DeletedTorrentResponse {
            torrent_id: torrent_listing.torrent_id,
            info_hash: torrent_listing.info_hash,
        })
    }

    /// Get torrent info from the Index
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * Unable to get torrent ID.
    /// * Unable to get torrent listing from id.
    /// * Unable to get torrent category from id.
    /// * Unable to get torrent files from id.
    /// * Unable to get torrent info from id.
    /// * Unable to get torrent announce url(s) from id.
    pub async fn get_torrent_info(
        &self,
        info_hash: &InfoHash,
        opt_user_id: Option<UserId>,
    ) -> Result<TorrentResponse, ServiceError> {
        let torrent_listing = self.torrent_listing_generator.one_torrent_by_info_hash(info_hash).await?;

        let torrent_id = torrent_listing.torrent_id;

        let category = self.category_repository.get_by_id(&torrent_listing.category_id).await?;

        let mut torrent_response = TorrentResponse::from_listing(torrent_listing, category);

        // Add files

        torrent_response.files = self.torrent_file_repository.get_by_torrent_id(&torrent_id).await?;

        if torrent_response.files.len() == 1 {
            let torrent_info = self.torrent_info_repository.get_by_info_hash(info_hash).await?;

            torrent_response
                .files
                .iter_mut()
                .for_each(|v| v.path = vec![torrent_info.name.to_string()]);
        }

        // Add trackers

        torrent_response.trackers = self.torrent_announce_url_repository.get_by_torrent_id(&torrent_id).await?;

        let tracker_url = self.get_tracker_url().await;

        // add tracker url
        match opt_user_id {
            Some(user_id) => {
                // if no user owned tracker key can be found, use default tracker url
                let personal_announce_url = self
                    .tracker_service
                    .get_personal_announce_url(user_id)
                    .await
                    .unwrap_or(tracker_url);
                // add personal tracker url to front of vec
                torrent_response.trackers.insert(0, personal_announce_url);
            }
            None => {
                torrent_response.trackers.insert(0, tracker_url);
            }
        }

        // Add magnet link

        // todo: extract a struct or function to build the magnet links
        let mut magnet = format!(
            "magnet:?xt=urn:btih:{}&dn={}",
            torrent_response.info_hash,
            urlencoding::encode(&torrent_response.title)
        );

        // Add trackers from torrent file to magnet link
        for tracker in &torrent_response.trackers {
            magnet.push_str(&format!("&tr={}", urlencoding::encode(tracker)));
        }

        torrent_response.magnet_link = magnet;

        // Get realtime seeders and leechers
        if let Ok(torrent_info) = self
            .tracker_statistics_importer
            .import_torrent_statistics(torrent_response.torrent_id, &torrent_response.info_hash)
            .await
        {
            torrent_response.seeders = torrent_info.seeders;
            torrent_response.leechers = torrent_info.leechers;
        }

        Ok(torrent_response)
    }

    /// It returns a list of torrents matching the search criteria.
    ///
    /// # Errors
    ///
    /// Returns a `ServiceError::DatabaseError` if the database query fails.
    pub async fn generate_torrent_info_listing(&self, request: &ListingRequest) -> Result<TorrentsResponse, ServiceError> {
        let torrent_listing_specification = self.listing_specification_from_user_request(request).await;

        let torrents_response = self
            .torrent_listing_generator
            .generate_listing(&torrent_listing_specification)
            .await?;

        Ok(torrents_response)
    }

    /// It converts the user listing request into an internal listing
    /// specification.
    async fn listing_specification_from_user_request(&self, request: &ListingRequest) -> ListingSpecification {
        let settings = self.configuration.settings.read().await;
        let default_torrent_page_size = settings.api.default_torrent_page_size;
        let max_torrent_page_size = settings.api.max_torrent_page_size;
        drop(settings);

        let sort = request.sort.unwrap_or(Sorting::UploadedDesc);
        let page = request.page.unwrap_or(0);
        let page_size = request.page_size.unwrap_or(default_torrent_page_size);

        // Guard that page size does not exceed the maximum
        let max_torrent_page_size = max_torrent_page_size;
        let page_size = if page_size > max_torrent_page_size {
            max_torrent_page_size
        } else {
            page_size
        };

        let offset = u64::from(page * u32::from(page_size));

        let categories = request.categories.as_csv::<String>().unwrap_or(None);

        ListingSpecification {
            search: request.search.clone(),
            categories,
            sort,
            offset,
            page_size,
        }
    }

    /// Update the torrent info on the Index.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// * Unable to get the user.
    /// * Unable to get listing from id.
    /// * Unable to update the torrent tile or description.
    /// * User does not have the permissions to update the torrent.
    pub async fn update_torrent_info(
        &self,
        info_hash: &InfoHash,
        title: &Option<String>,
        description: &Option<String>,
        user_id: &UserId,
    ) -> Result<TorrentResponse, ServiceError> {
        let updater = self.user_repository.get_compact_user(user_id).await?;

        let torrent_listing = self.torrent_listing_generator.one_torrent_by_info_hash(info_hash).await?;

        // Check if user is owner or administrator
        // todo: move this to an authorization service.
        if !(torrent_listing.uploader == updater.username || updater.administrator) {
            return Err(ServiceError::Unauthorized);
        }

        self.torrent_info_repository
            .update(&torrent_listing.torrent_id, title, description)
            .await?;

        let torrent_listing = self
            .torrent_listing_generator
            .one_torrent_by_torrent_id(&torrent_listing.torrent_id)
            .await?;

        let category = self.category_repository.get_by_id(&torrent_listing.category_id).await?;

        let torrent_response = TorrentResponse::from_listing(torrent_listing, category);

        Ok(torrent_response)
    }

    async fn get_tracker_url(&self) -> String {
        let settings = self.configuration.settings.read().await;
        settings.tracker.url.clone()
    }
}

pub struct DbTorrentRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbTorrentRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It finds the torrent by info-hash.
    ///
    /// # Errors
    ///
    /// This function will return an error there is a database error.
    pub async fn get_by_info_hash(&self, info_hash: &InfoHash) -> Result<Torrent, Error> {
        self.database.get_torrent_from_info_hash(info_hash).await
    }

    /// Inserts the entire torrent in the database.
    ///
    /// # Errors
    ///
    /// This function will return an error there is a database error.
    pub async fn add(&self, torrent_request: &TorrentRequest, user_id: UserId, category: Category) -> Result<TorrentId, Error> {
        self.database
            .insert_torrent_and_get_id(
                &torrent_request.torrent,
                user_id,
                category.category_id,
                &torrent_request.fields.title,
                &torrent_request.fields.description,
            )
            .await
    }

    /// Deletes the entire torrent in the database.
    ///
    /// # Errors
    ///
    /// This function will return an error there is a database error.
    pub async fn delete(&self, torrent_id: &TorrentId) -> Result<(), Error> {
        self.database.delete_torrent(*torrent_id).await
    }
}

pub struct DbTorrentInfoRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbTorrentInfoRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It finds the torrent info by info-hash.
    ///
    /// # Errors
    ///
    /// This function will return an error there is a database error.
    pub async fn get_by_info_hash(&self, info_hash: &InfoHash) -> Result<DbTorrentInfo, Error> {
        self.database.get_torrent_info_from_info_hash(info_hash).await
    }

    /// It updates the torrent title or/and description by torrent ID.
    ///
    /// # Errors
    ///
    /// This function will return an error there is a database error.
    pub async fn update(
        &self,
        torrent_id: &TorrentId,
        opt_title: &Option<String>,
        opt_description: &Option<String>,
    ) -> Result<(), Error> {
        if let Some(title) = &opt_title {
            self.database.update_torrent_title(*torrent_id, title).await?;
        }

        if let Some(description) = &opt_description {
            self.database.update_torrent_description(*torrent_id, description).await?;
        }

        Ok(())
    }
}

pub struct DbTorrentFileRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbTorrentFileRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It finds the torrent files by torrent id
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_by_torrent_id(&self, torrent_id: &TorrentId) -> Result<Vec<TorrentFile>, Error> {
        self.database.get_torrent_files_from_id(*torrent_id).await
    }
}

pub struct DbTorrentAnnounceUrlRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbTorrentAnnounceUrlRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It finds the announce URLs by torrent id
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_by_torrent_id(&self, torrent_id: &TorrentId) -> Result<Vec<String>, Error> {
        self.database
            .get_torrent_announce_urls_from_id(*torrent_id)
            .await
            .map(|v| v.into_iter().flatten().collect())
    }
}

pub struct DbTorrentListingGenerator {
    database: Arc<Box<dyn Database>>,
}

impl DbTorrentListingGenerator {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It finds the torrent listing by info-hash
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn one_torrent_by_info_hash(&self, info_hash: &InfoHash) -> Result<TorrentListing, Error> {
        self.database.get_torrent_listing_from_info_hash(info_hash).await
    }

    /// It finds the torrent listing by torrent ID.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn one_torrent_by_torrent_id(&self, torrent_id: &TorrentId) -> Result<TorrentListing, Error> {
        self.database.get_torrent_listing_from_id(*torrent_id).await
    }

    /// It finds the torrent listing by torrent ID.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn generate_listing(&self, specification: &ListingSpecification) -> Result<TorrentsResponse, Error> {
        self.database
            .get_torrents_search_sorted_paginated(
                &specification.search,
                &specification.categories,
                &specification.sort,
                specification.offset,
                specification.page_size,
            )
            .await
    }
}
