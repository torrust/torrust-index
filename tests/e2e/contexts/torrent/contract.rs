//! API contract for `torrent` context.

/*
todo:

Download torrent file:

- It should allow authenticated users to download a torrent with a personal tracker url

Delete torrent:

- After deleting a torrent, it should be removed from the tracker whitelist

Get torrent info:

- The torrent info:
    - should contain the tracker URL
        - If no user owned tracker key can be found, it should use the default tracker url
        - If    user owned tracker key can be found, it should use the personal tracker url
    - should contain the magnet link with the trackers from the torrent file
    - should contain realtime seeders and leechers from the tracker
*/

mod for_guests {
    use torrust_index_backend::utils::parse_torrent::decode_torrent;

    use crate::common::contexts::category::fixtures::software_predefined_category_id;
    use crate::common::contexts::torrent::asserts::assert_expected_torrent_details;
    use crate::common::contexts::torrent::responses::{
        Category, File, TorrentDetails, TorrentDetailsResponse, TorrentListResponse,
    };
    use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
    use crate::e2e::contexts::user::steps::logged_in_user;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_allow_guests_to_get_torrents() {
        let uploader = logged_in_user().await;
        let (_test_torrent, indexed_torrent) = upload_random_torrent_to_index(&uploader).await;

        let client = TestEnv::default().unauthenticated_client();

        let response = client.get_torrents().await;

        let torrent_list_response: TorrentListResponse = serde_json::from_str(&response.body).unwrap();

        assert!(torrent_list_response.data.total > 0);
        assert!(torrent_list_response.data.contains(indexed_torrent.torrent_id));
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_allow_guests_to_get_torrent_details_searching_by_id() {
        let uploader = logged_in_user().await;
        let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

        let client = TestEnv::default().unauthenticated_client();

        let response = client.get_torrent(uploaded_torrent.torrent_id).await;

        let torrent_details_response: TorrentDetailsResponse = serde_json::from_str(&response.body).unwrap();

        let expected_torrent = TorrentDetails {
            torrent_id: uploaded_torrent.torrent_id,
            uploader: uploader.username,
            info_hash: test_torrent.file_info.info_hash.to_uppercase(),
            title: test_torrent.index_info.title.clone(),
            description: test_torrent.index_info.description,
            category: Category {
                category_id: software_predefined_category_id(),
                name: test_torrent.index_info.category,
                num_torrents: 19, // Ignored in assertion
            },
            upload_date: "2023-04-27 07:56:08".to_string(), // Ignored in assertion
            file_size: test_torrent.file_info.content_size,
            seeders: 0,
            leechers: 0,
            files: vec![File {
                path: vec![test_torrent.file_info.files[0].clone()],
                // Using one file torrent for testing: content_size = first file size
                length: test_torrent.file_info.content_size,
                md5sum: None,
            }],
            // code-review: why is this duplicated?
            trackers: vec!["udp://tracker:6969".to_string(), "udp://tracker:6969".to_string()],
            magnet_link: format!(
                // cspell:disable-next-line
                "magnet:?xt=urn:btih:{}&dn={}&tr=udp%3A%2F%2Ftracker%3A6969&tr=udp%3A%2F%2Ftracker%3A6969",
                test_torrent.file_info.info_hash.to_uppercase(),
                test_torrent.index_info.title
            ),
        };

        assert_expected_torrent_details(&torrent_details_response.data, &expected_torrent);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_allow_guests_to_download_a_torrent_file_searching_by_id() {
        let uploader = logged_in_user().await;
        let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

        let client = TestEnv::default().unauthenticated_client();

        let response = client.download_torrent(uploaded_torrent.torrent_id).await;

        let torrent = decode_torrent(&response.bytes).unwrap();
        let mut expected_torrent = decode_torrent(&test_torrent.index_info.torrent_file.contents).unwrap();

        // code-review: The backend does not generate exactly the same torrent
        // that was uploaded and created by the `imdl` command-line tool.
        // So we need to update the expected torrent to match the one generated
        // by the backend. For some of them it makes sense (`announce`  and `announce_list`),
        // for others it does not.
        expected_torrent.info.private = Some(0);
        expected_torrent.announce = Some("udp://tracker:6969".to_string());
        expected_torrent.encoding = None;
        expected_torrent.announce_list = Some(vec![vec!["udp://tracker:6969".to_string()]]);
        expected_torrent.creation_date = None;
        expected_torrent.created_by = None;

        assert_eq!(torrent, expected_torrent);
        assert!(response.is_bittorrent_and_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_not_allow_guests_to_delete_torrents() {
        let uploader = logged_in_user().await;
        let (_test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

        let client = TestEnv::default().unauthenticated_client();

        let response = client.delete_torrent(uploaded_torrent.torrent_id).await;

        assert_eq!(response.status, 401);
    }
}

mod for_authenticated_users {

    use crate::common::contexts::torrent::fixtures::random_torrent;
    use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
    use crate::common::contexts::torrent::responses::UploadedTorrentResponse;
    use crate::e2e::contexts::user::steps::logged_in_user;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_allow_authenticated_users_to_upload_new_torrents() {
        let uploader = logged_in_user().await;
        let client = TestEnv::default().authenticated_client(&uploader.token);

        let test_torrent = random_torrent();

        let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

        let response = client.upload_torrent(form.into()).await;

        let _uploaded_torrent_response: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();

        // code-review: the response only returns the torrent autoincrement ID
        // generated by the DB. So we can't assert that the torrent was uploaded.
        // We could return the infohash.
        // We are going to use the infohash to get the torrent. See issue:
        // https://github.com/torrust/torrust-index-backend/issues/115

        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_not_allow_uploading_a_torrent_with_a_non_existing_category() {
        let uploader = logged_in_user().await;
        let client = TestEnv::default().authenticated_client(&uploader.token);

        let mut test_torrent = random_torrent();

        test_torrent.index_info.category = "non-existing-category".to_string();

        let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

        let response = client.upload_torrent(form.into()).await;

        assert_eq!(response.status, 400);
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_not_allow_uploading_a_torrent_with_a_title_that_already_exists() {
        let uploader = logged_in_user().await;
        let client = TestEnv::default().authenticated_client(&uploader.token);

        // Upload the first torrent
        let first_torrent = random_torrent();
        let first_torrent_title = first_torrent.index_info.title.clone();
        let form: UploadTorrentMultipartForm = first_torrent.index_info.into();
        let _response = client.upload_torrent(form.into()).await;

        // Upload the second torrent with the same title as the first one
        let mut second_torrent = random_torrent();
        second_torrent.index_info.title = first_torrent_title;
        let form: UploadTorrentMultipartForm = second_torrent.index_info.into();
        let response = client.upload_torrent(form.into()).await;

        assert_eq!(response.status, 400);
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "e2e-tests"), ignore)]
    async fn it_should_not_allow_uploading_a_torrent_with_a_infohash_that_already_exists() {
        let uploader = logged_in_user().await;
        let client = TestEnv::default().authenticated_client(&uploader.token);

        // Upload the first torrent
        let first_torrent = random_torrent();
        let mut first_torrent_clone = first_torrent.clone();
        let first_torrent_title = first_torrent.index_info.title.clone();
        let form: UploadTorrentMultipartForm = first_torrent.index_info.into();
        let _response = client.upload_torrent(form.into()).await;

        // Upload the second torrent with the same infohash as the first one.
        // We need to change the title otherwise the torrent will be rejected
        // because of the duplicate title.
        first_torrent_clone.index_info.title = format!("{}-clone", first_torrent_title);
        let form: UploadTorrentMultipartForm = first_torrent_clone.index_info.into();
        let response = client.upload_torrent(form.into()).await;

        assert_eq!(response.status, 400);
    }

    mod and_non_admins {
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::logged_in_user;
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        #[cfg_attr(not(feature = "e2e-tests"), ignore)]
        async fn it_should_not_allow_non_admins_to_delete_torrents() {
            let uploader = logged_in_user().await;
            let (_test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

            let client = TestEnv::default().authenticated_client(&uploader.token);

            let response = client.delete_torrent(uploaded_torrent.torrent_id).await;

            assert_eq!(response.status, 403);
        }
    }

    mod and_torrent_owners {
        use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
        use crate::common::contexts::torrent::responses::UpdatedTorrentResponse;
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::logged_in_user;
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        #[cfg_attr(not(feature = "e2e-tests"), ignore)]
        async fn it_should_allow_torrent_owners_to_update_their_torrents() {
            let uploader = logged_in_user().await;
            let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

            let client = TestEnv::default().authenticated_client(&uploader.token);

            let new_title = format!("{}-new-title", test_torrent.index_info.title);
            let new_description = format!("{}-new-description", test_torrent.index_info.description);

            let response = client
                .update_torrent(
                    uploaded_torrent.torrent_id,
                    UpdateTorrentFrom {
                        title: Some(new_title.clone()),
                        description: Some(new_description.clone()),
                    },
                )
                .await;

            let updated_torrent_response: UpdatedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            let torrent = updated_torrent_response.data;

            assert_eq!(torrent.title, new_title);
            assert_eq!(torrent.description, new_description);
            assert!(response.is_json_and_ok());
        }
    }

    mod and_admins {
        use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
        use crate::common::contexts::torrent::responses::{DeletedTorrentResponse, UpdatedTorrentResponse};
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::{logged_in_admin, logged_in_user};
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        #[cfg_attr(not(feature = "e2e-tests"), ignore)]
        async fn it_should_allow_admins_to_delete_torrents_searching_by_id() {
            let uploader = logged_in_user().await;
            let (_test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

            let admin = logged_in_admin().await;
            let client = TestEnv::default().authenticated_client(&admin.token);

            let response = client.delete_torrent(uploaded_torrent.torrent_id).await;

            let deleted_torrent_response: DeletedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            assert_eq!(deleted_torrent_response.data.torrent_id, uploaded_torrent.torrent_id);
            assert!(response.is_json_and_ok());
        }

        #[tokio::test]
        #[cfg_attr(not(feature = "e2e-tests"), ignore)]
        async fn it_should_allow_admins_to_update_someone_elses_torrents() {
            let uploader = logged_in_user().await;
            let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader).await;

            let logged_in_admin = logged_in_admin().await;
            let client = TestEnv::default().authenticated_client(&logged_in_admin.token);

            let new_title = format!("{}-new-title", test_torrent.index_info.title);
            let new_description = format!("{}-new-description", test_torrent.index_info.description);

            let response = client
                .update_torrent(
                    uploaded_torrent.torrent_id,
                    UpdateTorrentFrom {
                        title: Some(new_title.clone()),
                        description: Some(new_description.clone()),
                    },
                )
                .await;

            let updated_torrent_response: UpdatedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            let torrent = updated_torrent_response.data;

            assert_eq!(torrent.title, new_title);
            assert_eq!(torrent.description, new_description);
            assert!(response.is_json_and_ok());
        }
    }
}
