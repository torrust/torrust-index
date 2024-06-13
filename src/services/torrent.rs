//! Torrent service.
use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

use super::authorization::{self, ACTION};
use super::category::DbCategoryRepository;
use crate::config::{Configuration, TrackerMode};
use crate::databases::database::{Database, Error, Sorting};
use crate::errors::ServiceError;
use crate::models::category::CategoryId;
use crate::models::info_hash::InfoHash;
use crate::models::response::{DeletedTorrentResponse, TorrentResponse, TorrentsResponse};
use crate::models::torrent::{Metadata, TorrentId, TorrentListing};
use crate::models::torrent_file::{DbTorrent, Torrent, TorrentFile};
use crate::models::torrent_tag::{TagId, TorrentTag};
use crate::models::user::UserId;
use crate::services::user::Repository;
use crate::tracker::statistics_importer::StatisticsImporter;
use crate::utils::parse_torrent::decode_and_validate_torrent_file;
use crate::{tracker, AsCSV};

pub struct Index {
    configuration: Arc<Configuration>,
    tracker_statistics_importer: Arc<StatisticsImporter>,
    tracker_service: Arc<tracker::service::Service>,
    user_repository: Arc<Box<dyn Repository>>,
    category_repository: Arc<DbCategoryRepository>,
    torrent_repository: Arc<DbTorrentRepository>,
    torrent_info_hash_repository: Arc<DbCanonicalInfoHashGroupRepository>,
    torrent_info_repository: Arc<DbTorrentInfoRepository>,
    torrent_file_repository: Arc<DbTorrentFileRepository>,
    torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
    torrent_tag_repository: Arc<DbTorrentTagRepository>,
    torrent_listing_generator: Arc<DbTorrentListingGenerator>,
    authorization_service: Arc<authorization::Service>,
}

pub struct AddTorrentRequest {
    pub title: String,
    pub description: String,
    pub category_name: String,
    pub tags: Vec<TagId>,
    pub torrent_buffer: Vec<u8>,
}

pub struct AddTorrentResponse {
    pub torrent_id: TorrentId,
    pub canonical_info_hash: String,
    pub info_hash: String,
}

/// User request to generate a torrent listing.
#[derive(Debug, Deserialize)]
pub struct ListingRequest {
    pub page_size: Option<u8>,
    pub page: Option<u32>,
    pub sort: Option<Sorting>,
    /// Expects comma separated string, eg: "?categories=movie,other,app"
    pub categories: Option<String>,
    /// Expects comma separated string, eg: "?tags=Linux,Ubuntu"
    pub tags: Option<String>,
    pub search: Option<String>,
}

/// Internal specification for torrent listings.
#[derive(Debug, Deserialize)]
pub struct ListingSpecification {
    pub search: Option<String>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
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
        user_repository: Arc<Box<dyn Repository>>,
        category_repository: Arc<DbCategoryRepository>,
        torrent_repository: Arc<DbTorrentRepository>,
        torrent_info_hash_repository: Arc<DbCanonicalInfoHashGroupRepository>,
        torrent_info_repository: Arc<DbTorrentInfoRepository>,
        torrent_file_repository: Arc<DbTorrentFileRepository>,
        torrent_announce_url_repository: Arc<DbTorrentAnnounceUrlRepository>,
        torrent_tag_repository: Arc<DbTorrentTagRepository>,
        torrent_listing_repository: Arc<DbTorrentListingGenerator>,
        authorization_service: Arc<authorization::Service>,
    ) -> Self {
        Self {
            configuration,
            tracker_statistics_importer,
            tracker_service,
            user_repository,
            category_repository,
            torrent_repository,
            torrent_info_hash_repository,
            torrent_info_repository,
            torrent_file_repository,
            torrent_announce_url_repository,
            torrent_tag_repository,
            torrent_listing_generator: torrent_listing_repository,
            authorization_service,
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
    /// * Torrent title is too short.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    ///
    /// * Unable to parse the torrent info-hash.
    pub async fn add_torrent(
        &self,
        add_torrent_req: AddTorrentRequest,
        user_id: UserId,
    ) -> Result<AddTorrentResponse, ServiceError> {
        // Guard that the users exists
        let _user = self.user_repository.get_compact(&user_id).await?;

        let metadata = self.validate_and_build_metadata(&add_torrent_req).await?;

        let (mut torrent, original_info_hash) = decode_and_validate_torrent_file(&add_torrent_req.torrent_buffer)?;

        self.customize_announcement_info_for(&mut torrent).await;

        self.canonical_info_hash_group_checks(&original_info_hash, &torrent.canonical_info_hash())
            .await?;

        let torrent_id = self
            .torrent_repository
            .add(&original_info_hash, &torrent, &metadata, user_id)
            .await?;

        // Synchronous secondary tasks

        // code-review: consider moving this to a background task
        self.import_torrent_statistics_from_tracker(torrent_id, &torrent.canonical_info_hash())
            .await;

        // We always whitelist the torrent on the tracker because
        // even if the tracker mode is `public` it could be changed to `private`
        // later on.
        //
        // code-review: maybe we should consider adding a new feature to
        // whitelist  all torrents from the admin panel if that change happens.
        if let Err(e) = self
            .tracker_service
            .whitelist_info_hash(torrent.canonical_info_hash_hex())
            .await
        {
            // If the torrent can't be whitelisted somehow, remove the torrent from database
            drop(self.torrent_repository.delete(&torrent_id).await);
            return Err(e.into());
        }

        // Build response

        Ok(AddTorrentResponse {
            torrent_id,
            canonical_info_hash: torrent.canonical_info_hash_hex(),
            info_hash: original_info_hash.to_string(),
        })
    }

    async fn validate_and_build_metadata(&self, add_torrent_req: &AddTorrentRequest) -> Result<Metadata, ServiceError> {
        if add_torrent_req.category_name.is_empty() {
            return Err(ServiceError::MissingMandatoryMetadataFields);
        }

        let category = self
            .category_repository
            .get_by_name(&add_torrent_req.category_name)
            .await
            .map_err(|_| ServiceError::InvalidCategory)?;

        let metadata = Metadata::new(
            &add_torrent_req.title,
            &add_torrent_req.description,
            category.category_id,
            &add_torrent_req.tags,
        )?;

        Ok(metadata)
    }

    async fn canonical_info_hash_group_checks(
        &self,
        original_info_hash: &InfoHash,
        canonical_info_hash: &InfoHash,
    ) -> Result<(), ServiceError> {
        let original_info_hashes = self
            .torrent_info_hash_repository
            .get_canonical_info_hash_group(canonical_info_hash)
            .await?;

        if !original_info_hashes.is_empty() {
            // A previous torrent with the same canonical infohash has been uploaded before

            // Torrent with the same canonical infohash was already uploaded
            debug!("Canonical infohash found: {:?}", canonical_info_hash.to_hex_string());

            if let Some(original_info_hash) = original_info_hashes.find(original_info_hash) {
                // The exact original infohash was already uploaded
                debug!("Original infohash found: {:?}", original_info_hash.to_hex_string());

                return Err(ServiceError::OriginalInfoHashAlreadyExists);
            }

            // A new original infohash is being uploaded with a canonical infohash that already exists.
            debug!("Original infohash not found: {:?}", original_info_hash.to_hex_string());

            // Add the new associated original infohash to the canonical one.
            self.torrent_info_hash_repository
                .add_info_hash_to_canonical_info_hash_group(original_info_hash, canonical_info_hash)
                .await?;
            return Err(ServiceError::CanonicalInfoHashAlreadyExists);
        }

        // No other torrent with the same canonical infohash has been uploaded before
        Ok(())
    }

    async fn customize_announcement_info_for(&self, torrent: &mut Torrent) {
        let settings = self.configuration.settings.read().await;
        let tracker_url = settings.tracker.url.clone();
        torrent.set_announce_to(&tracker_url);
        torrent.reset_announce_list_if_private();
    }

    async fn import_torrent_statistics_from_tracker(&self, torrent_id: TorrentId, canonical_info_hash: &InfoHash) {
        drop(
            self.tracker_statistics_importer
                .import_torrent_statistics(torrent_id, &canonical_info_hash.to_hex_string())
                .await,
        );
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
        let tracker_mode = self.get_tracker_mode().await;

        // code-review: should we remove all tracker URLs in the `announce_list`
        // when the tracker is not open?

        if tracker_mode.is_open() {
            torrent.include_url_as_main_tracker(&tracker_url);
        } else if let Some(authenticated_user_id) = opt_user_id {
            let personal_announce_url = self.tracker_service.get_personal_announce_url(authenticated_user_id).await?;
            torrent.include_url_as_main_tracker(&personal_announce_url);
        } else {
            torrent.include_url_as_main_tracker(&tracker_url);
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
        self.authorization_service
            .authorize(ACTION::DeleteTorrent, Some(*user_id))
            .await?;

        let torrent_listing = self.torrent_listing_generator.one_torrent_by_info_hash(info_hash).await?;

        self.torrent_repository.delete(&torrent_listing.torrent_id).await?;

        // Remove info-hash from tracker whitelist
        // todo: handle the error when the tracker is offline or not well configured.
        let _unused = self
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

        let torrent_response = self
            .build_full_torrent_response(torrent_listing, info_hash, opt_user_id)
            .await?;

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
        let page_size = if page_size > max_torrent_page_size {
            max_torrent_page_size
        } else {
            page_size
        };

        let offset = u64::from(page * u32::from(page_size));

        let categories = request.categories.as_csv::<String>().unwrap_or(None);

        let tags = request.tags.as_csv::<String>().unwrap_or(None);

        ListingSpecification {
            search: request.search.clone(),
            categories,
            tags,
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
        category_id: &Option<CategoryId>,
        tags: &Option<Vec<TagId>>,
        user_id: &UserId,
    ) -> Result<TorrentResponse, ServiceError> {
        let updater = self.user_repository.get_compact(user_id).await?;

        let torrent_listing = self.torrent_listing_generator.one_torrent_by_info_hash(info_hash).await?;

        // Check if user is owner or administrator
        // todo: move this to an authorization service.
        if !(torrent_listing.uploader == updater.username || updater.administrator) {
            return Err(ServiceError::Unauthorized);
        }

        self.torrent_info_repository
            .update(&torrent_listing.torrent_id, title, description, category_id, tags)
            .await?;

        let torrent_listing = self
            .torrent_listing_generator
            .one_torrent_by_torrent_id(&torrent_listing.torrent_id)
            .await?;

        let torrent_response = self.build_short_torrent_response(torrent_listing, info_hash).await?;

        Ok(torrent_response)
    }

    async fn get_tracker_url(&self) -> Url {
        let settings = self.configuration.settings.read().await;
        settings.tracker.url.clone()
    }

    async fn get_tracker_mode(&self) -> TrackerMode {
        let settings = self.configuration.settings.read().await;
        settings.tracker.mode.clone()
    }

    async fn build_short_torrent_response(
        &self,
        torrent_listing: TorrentListing,
        info_hash: &InfoHash,
    ) -> Result<TorrentResponse, ServiceError> {
        let category = match torrent_listing.category_id {
            Some(category_id) => Some(self.category_repository.get_by_id(&category_id).await?),
            None => None,
        };

        let canonical_info_hash_group = self
            .torrent_info_hash_repository
            .get_canonical_info_hash_group(info_hash)
            .await?;

        Ok(TorrentResponse::from_listing(
            torrent_listing,
            category,
            &canonical_info_hash_group,
        ))
    }

    async fn build_full_torrent_response(
        &self,
        torrent_listing: TorrentListing,
        info_hash: &InfoHash,
        opt_user_id: Option<UserId>,
    ) -> Result<TorrentResponse, ServiceError> {
        let torrent_id: i64 = torrent_listing.torrent_id;

        let mut torrent_response = self.build_short_torrent_response(torrent_listing, info_hash).await?;

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

        // code-review: duplicate logic. We have to check the same in the
        // download torrent file endpoint. Here he have only one list of tracker
        // like the `announce_list` in the torrent file.

        torrent_response.trackers = self.torrent_announce_url_repository.get_by_torrent_id(&torrent_id).await?;

        let tracker_url = self.get_tracker_url().await;
        let tracker_mode = self.get_tracker_mode().await;

        if tracker_mode.is_open() {
            torrent_response.include_url_as_main_tracker(&tracker_url);
        } else {
            // Add main tracker URL
            match opt_user_id {
                Some(user_id) => {
                    let personal_announce_url = self.tracker_service.get_personal_announce_url(user_id).await?;

                    torrent_response.include_url_as_main_tracker(&personal_announce_url);
                }
                None => {
                    torrent_response.include_url_as_main_tracker(&tracker_url);
                }
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

        torrent_response.tags = self.torrent_tag_repository.get_tags_for_torrent(&torrent_id).await?;

        Ok(torrent_response)
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
    pub async fn add(
        &self,
        original_info_hash: &InfoHash,
        torrent: &Torrent,
        metadata: &Metadata,
        user_id: UserId,
    ) -> Result<TorrentId, Error> {
        self.database
            .insert_torrent_and_get_id(original_info_hash, torrent, user_id, metadata)
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

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentInfoHash {
    pub info_hash: String,
    pub canonical_info_hash: String,
    pub original_is_known: bool,
}

/// All the infohashes associated to a canonical one.
///
/// When you upload a torrent the info-hash migth change because the Index
/// remove the non-standard fields in the `info` dictionary. That makes the
/// infohash change. The canonical infohash is the resulting infohash.
/// This function returns the original infohashes of a canonical infohash.
///
/// The relationship is 1 canonical infohash -> N original infohashes.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalInfoHashGroup {
    pub canonical_info_hash: InfoHash,
    /// The list of original infohashes associated to the canonical one.
    pub original_info_hashes: Vec<InfoHash>,
}
pub struct DbCanonicalInfoHashGroupRepository {
    database: Arc<Box<dyn Database>>,
}

impl CanonicalInfoHashGroup {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.original_info_hashes.is_empty()
    }

    #[must_use]
    pub fn find(&self, original_info_hash: &InfoHash) -> Option<&InfoHash> {
        self.original_info_hashes.iter().find(|&hash| *hash == *original_info_hash)
    }
}

impl DbCanonicalInfoHashGroupRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It returns all the infohashes associated to the canonical one.
    ///
    /// # Errors
    ///
    /// This function will return an error there is a database error.
    ///
    /// # Errors
    ///
    /// Returns an error is there was a problem with the database.
    pub async fn get_canonical_info_hash_group(&self, info_hash: &InfoHash) -> Result<CanonicalInfoHashGroup, Error> {
        self.database.get_torrent_canonical_info_hash_group(info_hash).await
    }

    /// It returns the list of all infohashes producing the same canonical
    /// infohash.
    ///
    /// If the original infohash was unknown, it returns the canonical infohash.
    ///
    /// # Errors
    ///
    /// Returns an error is there was a problem with the database.
    pub async fn find_canonical_info_hash_for(&self, info_hash: &InfoHash) -> Result<Option<InfoHash>, Error> {
        self.database.find_canonical_info_hash_for(info_hash).await
    }

    /// It returns the list of all infohashes producing the same canonical
    /// infohash.
    ///
    /// If the original infohash was unknown, it returns the canonical infohash.
    ///
    /// # Errors
    ///
    /// Returns an error is there was a problem with the database.
    pub async fn add_info_hash_to_canonical_info_hash_group(
        &self,
        original_info_hash: &InfoHash,
        canonical_info_hash: &InfoHash,
    ) -> Result<(), Error> {
        self.database
            .add_info_hash_to_canonical_info_hash_group(original_info_hash, canonical_info_hash)
            .await
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
    pub async fn get_by_info_hash(&self, info_hash: &InfoHash) -> Result<DbTorrent, Error> {
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
        opt_category_id: &Option<CategoryId>,
        opt_tags: &Option<Vec<TagId>>,
    ) -> Result<(), Error> {
        if let Some(title) = &opt_title {
            self.database.update_torrent_title(*torrent_id, title).await?;
        }

        if let Some(description) = &opt_description {
            self.database.update_torrent_description(*torrent_id, description).await?;
        }

        if let Some(category_id) = &opt_category_id {
            self.database.update_torrent_category(*torrent_id, *category_id).await?;
        }

        if let Some(tags) = opt_tags {
            let mut current_tags: Vec<TagId> = self
                .database
                .get_tags_for_torrent_id(*torrent_id)
                .await?
                .iter()
                .map(|tag| tag.tag_id)
                .collect();

            let mut new_tags = tags.clone();

            current_tags.sort_unstable();
            new_tags.sort_unstable();

            if new_tags != current_tags {
                self.database.delete_all_torrent_tag_links(*torrent_id).await?;
                self.database.add_torrent_tag_links(*torrent_id, tags).await?;
            }
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

pub struct DbTorrentTagRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbTorrentTagRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It adds a new torrent tag link.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn link_torrent_to_tag(&self, torrent_id: &TorrentId, tag_id: &TagId) -> Result<(), Error> {
        self.database.add_torrent_tag_link(*torrent_id, *tag_id).await
    }

    /// It adds multiple torrent tag links at once.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn link_torrent_to_tags(&self, torrent_id: &TorrentId, tag_ids: &[TagId]) -> Result<(), Error> {
        self.database.add_torrent_tag_links(*torrent_id, tag_ids).await
    }

    /// It returns all the tags linked to a certain torrent ID.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_tags_for_torrent(&self, torrent_id: &TorrentId) -> Result<Vec<TorrentTag>, Error> {
        self.database.get_tags_for_torrent_id(*torrent_id).await
    }

    /// It removes a torrent tag link.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn unlink_torrent_from_tag(&self, torrent_id: &TorrentId, tag_id: &TagId) -> Result<(), Error> {
        self.database.delete_torrent_tag_link(*torrent_id, *tag_id).await
    }

    /// It removes all tags for a certain torrent.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn unlink_all_tags_for_torrent(&self, torrent_id: &TorrentId) -> Result<(), Error> {
        self.database.delete_all_torrent_tag_links(*torrent_id).await
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
                &specification.tags,
                &specification.sort,
                specification.offset,
                specification.page_size,
            )
            .await
    }
}
