//! API contract for `torrent` context.

/*
todo:

Delete torrent:

- After deleting a torrent, it should be removed from the tracker whitelist

Get torrent info:

- The torrent info:
    - should contain the magnet link with the trackers from the torrent file
    - should contain realtime seeders and leechers from the tracker
*/

mod for_guests {

    use torrust_index_backend::utils::parse_torrent::decode_torrent;
    use torrust_index_backend::web::api;
    use uuid::Uuid;

    use crate::common::client::Client;
    use crate::common::contexts::category::fixtures::software_predefined_category_id;
    use crate::common::contexts::torrent::asserts::assert_expected_torrent_details;
    use crate::common::contexts::torrent::fixtures::{random_torrent, TestTorrent};
    use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
    use crate::common::contexts::torrent::requests::InfoHash;
    use crate::common::contexts::torrent::responses::{
        Category, File, TorrentDetails, TorrentDetailsResponse, TorrentListResponse,
    };
    use crate::common::http::{Query, QueryParam};
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::torrent::steps::{upload_random_torrent_to_index, upload_test_torrent};
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

    #[tokio::test]
    async fn it_should_allow_guests_to_get_torrents() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let uploader = new_logged_in_user(&env).await;
        let (_test_torrent, _indexed_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

        let response = client.get_torrents(Query::empty()).await;

        let torrent_list_response: TorrentListResponse = serde_json::from_str(&response.body).unwrap();

        assert!(torrent_list_response.data.total > 0);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_allow_to_get_torrents_with_pagination() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let uploader = new_logged_in_user(&env).await;

        // Given we insert two torrents
        let (_test_torrent, _indexed_torrent) = upload_random_torrent_to_index(&uploader, &env).await;
        let (_test_torrent, _indexed_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        // When we request only one torrent per page
        let response = client
            .get_torrents(Query::with_params([QueryParam::new("page_size", "1")].to_vec()))
            .await;

        let torrent_list_response: TorrentListResponse = serde_json::from_str(&response.body).unwrap();

        // Then we should have only one torrent per page
        assert_eq!(torrent_list_response.data.results.len(), 1);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_allow_to_limit_the_number_of_torrents_per_request() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let uploader = new_logged_in_user(&env).await;

        let max_torrent_page_size = 30;

        // Given we insert one torrent more than the page size limit
        for _ in 0..max_torrent_page_size {
            let (_test_torrent, _indexed_torrent) = upload_random_torrent_to_index(&uploader, &env).await;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        // When we request more torrents than the page size limit
        let response = client
            .get_torrents(Query::with_params(
                [QueryParam::new("page_size", &format!("{}", (max_torrent_page_size + 1)))].to_vec(),
            ))
            .await;

        let torrent_list_response: TorrentListResponse = serde_json::from_str(&response.body).unwrap();

        // Then we should get only the page size limit
        assert_eq!(torrent_list_response.data.results.len(), max_torrent_page_size);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_return_a_default_amount_of_torrents_per_request_if_no_page_size_is_provided() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let uploader = new_logged_in_user(&env).await;

        let default_torrent_page_size = 10;

        // Given we insert one torrent more than the default page size
        for _ in 0..default_torrent_page_size {
            let (_test_torrent, _indexed_torrent) = upload_random_torrent_to_index(&uploader, &env).await;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        // When we request more torrents than the default page size limit
        let response = client.get_torrents(Query::empty()).await;

        let torrent_list_response: TorrentListResponse = serde_json::from_str(&response.body).unwrap();

        // Then we should get only the default number of torrents per page
        assert_eq!(torrent_list_response.data.results.len(), default_torrent_page_size);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_allow_guests_to_get_torrent_details_searching_by_info_hash() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let uploader = new_logged_in_user(&env).await;
        let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

        let response = client.get_torrent(&test_torrent.file_info_hash()).await;

        let torrent_details_response: TorrentDetailsResponse = serde_json::from_str(&response.body).unwrap();

        let tracker_url = env.server_settings().unwrap().tracker.url;
        let encoded_tracker_url = urlencoding::encode(&tracker_url);

        let expected_torrent = TorrentDetails {
            torrent_id: uploaded_torrent.torrent_id,
            uploader: uploader.username,
            info_hash: test_torrent.file_info.info_hash.to_lowercase(),
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
            // code-review: why is this duplicated? It seems that is adding the
            // same tracker twice because first ti adds all trackers and then
            // it adds the tracker with the personal announce url, if the user
            // is logged in. If the user is not logged in, it adds the default
            // tracker again, and it ends up with two trackers.
            trackers: vec![tracker_url.clone(), tracker_url.clone()],
            magnet_link: format!(
                // cspell:disable-next-line
                "magnet:?xt=urn:btih:{}&dn={}&tr={}&tr={}",
                test_torrent.file_info.info_hash.to_lowercase(),
                urlencoding::encode(&test_torrent.index_info.title),
                encoded_tracker_url,
                encoded_tracker_url
            ),
            tags: vec![],
            name: test_torrent.index_info.name.clone(),
            comment: test_torrent.file_info.comment.clone(),
        };

        assert_expected_torrent_details(&torrent_details_response.data, &expected_torrent);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_allow_guests_to_find_torrent_details_using_a_non_canonical_info_hash() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let uploader = new_logged_in_user(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

        // Sample data needed to build two torrents with the same canonical info-hash.
        // Those torrents belong to the same Canonical Infohash Group.
        let id = Uuid::new_v4();
        let title = format!("title-{id}");
        let file_contents = "data".to_string();

        // Upload the first torrent
        let mut first_torrent = TestTorrent::with_custom_info_dict_field(id, &file_contents, "custom 01");
        first_torrent.index_info.title = title.clone();

        let first_torrent_canonical_info_hash = upload_test_torrent(&client, &first_torrent)
            .await
            .expect("first torrent should be uploaded");

        // Upload the second torrent with the same canonical info-hash
        let mut second_torrent = TestTorrent::with_custom_info_dict_field(id, &file_contents, "custom 02");
        second_torrent.index_info.title = format!("{title}-clone");

        let _result = upload_test_torrent(&client, &second_torrent).await;

        // Get torrent details using the non-canonical info-hash (second torrent info-hash)
        let response = client.get_torrent(&second_torrent.file_info_hash()).await;
        let torrent_details_response: TorrentDetailsResponse = serde_json::from_str(&response.body).unwrap();

        // The returned torrent info should be the same as the first torrent
        assert_eq!(response.status, 200);
        assert_eq!(
            torrent_details_response.data.info_hash,
            first_torrent_canonical_info_hash.to_hex_string()
        );
    }

    mod it_should_allow_guests_to_download_a_torrent_file_searching_by_info_hash {

        use torrust_index_backend::utils::parse_torrent::{calculate_info_hash, decode_torrent};
        use torrust_index_backend::web::api;

        use crate::common::client::Client;
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::torrent::asserts::canonical_torrent_for;
        use crate::e2e::web::api::v1::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

        #[tokio::test]
        async fn returning_a_bittorrent_binary_ok_response() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());
            let uploader = new_logged_in_user(&env).await;

            // Upload
            let (test_torrent, _torrent_listed_in_index) = upload_random_torrent_to_index(&uploader, &env).await;

            // Download
            let response = client.download_torrent(&test_torrent.file_info_hash()).await;

            assert!(response.is_a_bit_torrent_file());
        }

        #[tokio::test]
        async fn the_downloaded_torrent_should_keep_the_same_info_hash_if_the_torrent_does_not_have_non_standard_fields_in_the_info_dict(
        ) {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());
            let uploader = new_logged_in_user(&env).await;

            // Upload
            let (test_torrent, _torrent_listed_in_index) = upload_random_torrent_to_index(&uploader, &env).await;

            // Download
            let response = client.download_torrent(&test_torrent.file_info_hash()).await;

            let downloaded_torrent_info_hash =
                calculate_info_hash(&response.bytes).expect("failed to calculate info-hash of the downloaded torrent");

            assert_eq!(
                downloaded_torrent_info_hash.to_hex_string(),
                test_torrent.file_info_hash(),
                "downloaded torrent info-hash does not match uploaded torrent info-hash"
            );
        }

        #[tokio::test]
        async fn the_downloaded_torrent_should_be_the_canonical_version_of_the_uploaded_one() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());
            let uploader = new_logged_in_user(&env).await;

            // Upload
            let (test_torrent, _torrent_listed_in_index) = upload_random_torrent_to_index(&uploader, &env).await;

            let uploaded_torrent =
                decode_torrent(&test_torrent.index_info.torrent_file.contents).expect("could not decode uploaded torrent");

            // Download
            let response = client.download_torrent(&test_torrent.file_info_hash()).await;

            let downloaded_torrent = decode_torrent(&response.bytes).expect("could not decode downloaded torrent");

            let expected_downloaded_torrent = canonical_torrent_for(uploaded_torrent, &env, &None).await;

            assert_eq!(downloaded_torrent, expected_downloaded_torrent);
        }
    }

    #[tokio::test]
    async fn it_should_allow_guests_to_download_a_torrent_using_a_non_canonical_info_hash() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let uploader = new_logged_in_user(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

        // Sample data needed to build two torrents with the same canonical info-hash.
        // Those torrents belong to the same Canonical Infohash Group.
        let id = Uuid::new_v4();
        let title = format!("title-{id}");
        let file_contents = "data".to_string();

        // Upload the first torrent
        let mut first_torrent = TestTorrent::with_custom_info_dict_field(id, &file_contents, "custom 01");
        first_torrent.index_info.title = title.clone();

        let first_torrent_canonical_info_hash = upload_test_torrent(&client, &first_torrent)
            .await
            .expect("first torrent should be uploaded");

        // Upload the second torrent with the same canonical info-hash
        let mut second_torrent = TestTorrent::with_custom_info_dict_field(id, &file_contents, "custom 02");
        second_torrent.index_info.title = format!("{title}-clone");

        let _result = upload_test_torrent(&client, &second_torrent).await;

        // Download the torrent using the non-canonical info-hash (second torrent info-hash)
        let response = client.download_torrent(&second_torrent.file_info_hash()).await;

        let torrent = decode_torrent(&response.bytes).expect("could not decode downloaded torrent");

        // The returned torrent info-hash should be the same as the first torrent
        assert_eq!(response.status, 200);
        assert_eq!(
            torrent.canonical_info_hash_hex(),
            first_torrent_canonical_info_hash.to_hex_string()
        );
    }

    #[tokio::test]
    async fn it_should_return_a_not_found_response_trying_to_get_the_torrent_info_for_a_non_existing_torrent() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let non_existing_info_hash: InfoHash = "443c7602b4fde83d1154d6d9da48808418b181b6".to_string();

        let response = client.get_torrent(&non_existing_info_hash).await;

        assert_eq!(response.status, 404);
    }

    #[tokio::test]
    async fn it_should_return_a_not_found_trying_to_download_a_non_existing_torrent() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let non_existing_info_hash: InfoHash = "443c7602b4fde83d1154d6d9da48808418b181b6".to_string();

        let response = client.download_torrent(&non_existing_info_hash).await;

        assert_eq!(response.status, 404);
    }

    #[tokio::test]
    async fn it_should_not_allow_guests_to_upload_torrents() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let test_torrent = random_torrent();

        let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

        let response = client.upload_torrent(form.into()).await;

        assert_eq!(response.status, 401);
    }

    #[tokio::test]
    async fn it_should_not_allow_guests_to_delete_torrents() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let uploader = new_logged_in_user(&env).await;
        let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

        let response = client.delete_torrent(&test_torrent.file_info_hash()).await;

        assert_eq!(response.status, 401);
    }
}

mod for_authenticated_users {

    use torrust_index_backend::utils::parse_torrent::decode_torrent;
    use torrust_index_backend::web::api;

    use crate::common::client::Client;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::torrent::asserts::{build_announce_url, get_user_tracker_key};
    use crate::e2e::web::api::v1::contexts::torrent::steps::upload_random_torrent_to_index;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

    mod uploading_a_torrent {

        use torrust_index_backend::web::api;
        use uuid::Uuid;

        use crate::common::asserts::assert_json_error_response;
        use crate::common::client::Client;
        use crate::common::contexts::torrent::fixtures::{random_torrent, TestTorrent};
        use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
        use crate::common::contexts::torrent::responses::UploadedTorrentResponse;
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

        #[tokio::test]
        async fn it_should_allow_authenticated_users_to_upload_new_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let test_torrent = random_torrent();
            let info_hash = test_torrent.file_info_hash().clone();

            let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

            let response = client.upload_torrent(form.into()).await;

            let uploaded_torrent_response: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            assert_eq!(
                uploaded_torrent_response.data.info_hash.to_lowercase(),
                info_hash.to_lowercase()
            );
            assert!(response.is_json_and_ok());
        }

        mod it_should_guard_that_torrent_metadata {
            use torrust_index_backend::web::api;

            use crate::common::client::Client;
            use crate::common::contexts::torrent::fixtures::random_torrent;
            use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
            use crate::e2e::environment::TestEnv;
            use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

            #[tokio::test]
            async fn contains_a_non_empty_category_name() {
                let mut env = TestEnv::new();
                env.start(api::Version::V1).await;

                let uploader = new_logged_in_user(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let mut test_torrent = random_torrent();

                test_torrent.index_info.category = String::new();

                let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

                let response = client.upload_torrent(form.into()).await;

                assert_eq!(response.status, 400);
            }

            #[tokio::test]
            async fn contains_a_non_empty_title() {
                let mut env = TestEnv::new();
                env.start(api::Version::V1).await;

                let uploader = new_logged_in_user(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let mut test_torrent = random_torrent();

                test_torrent.index_info.title = String::new();

                let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

                let response = client.upload_torrent(form.into()).await;

                assert_eq!(response.status, 400);
            }

            #[tokio::test]
            async fn title_has_at_least_3_chars() {
                let mut env = TestEnv::new();
                env.start(api::Version::V1).await;

                let uploader = new_logged_in_user(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let mut test_torrent = random_torrent();

                test_torrent.index_info.title = "12".to_string();

                let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

                let response = client.upload_torrent(form.into()).await;

                assert_eq!(response.status, 400);
            }
        }

        mod it_should_guard_that_the_torrent_file {

            use torrust_index_backend::web::api;

            use crate::common::client::Client;
            use crate::common::contexts::torrent::fixtures::random_torrent;
            use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
            use crate::e2e::environment::TestEnv;
            use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

            #[tokio::test]
            async fn contains_a_bencoded_dictionary_with_the_info_key_in_order_to_calculate_the_original_info_hash() {
                let mut env = TestEnv::new();
                env.start(api::Version::V1).await;

                let uploader = new_logged_in_user(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let mut test_torrent = random_torrent();

                // Make the random torrent invalid by changing the bytes of the torrent file
                let minimal_bencoded_value = b"de".to_vec();
                test_torrent.index_info.torrent_file.contents = minimal_bencoded_value;

                let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

                let response = client.upload_torrent(form.into()).await;

                assert_eq!(response.status, 400);
            }

            #[tokio::test]
            async fn contains_a_valid_metainfo_file() {
                let mut env = TestEnv::new();
                env.start(api::Version::V1).await;

                let uploader = new_logged_in_user(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let mut test_torrent = random_torrent();

                // Make the random torrent invalid by changing the bytes of the torrent file.
                // It's a valid bencoded format but an invalid torrent. It contains
                // a `info` otherwise the test to validate the `info` key would fail.
                // cspell:disable-next-line
                let minimal_bencoded_value_with_info_key = b"d4:infod6:custom6:customee".to_vec();
                test_torrent.index_info.torrent_file.contents = minimal_bencoded_value_with_info_key;

                let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

                let response = client.upload_torrent(form.into()).await;

                assert_eq!(response.status, 400);
            }

            #[tokio::test]
            async fn pieces_key_has_a_length_that_is_a_multiple_of_20() {
                let mut env = TestEnv::new();
                env.start(api::Version::V1).await;

                let uploader = new_logged_in_user(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let mut test_torrent = random_torrent();

                // cspell:disable-next-line
                let torrent_with_19_pieces = b"d4:infod6:lengthi2e4:name42:torrent-with-invalid-pieces-key-length.txt12:piece lengthi16384e6:pieces19:\x3F\x78\x68\x50\xE3\x87\x55\x0F\xDA\xB8\x36\xED\x7E\x6D\xC8\x81\xDE\x23\x00ee";
                test_torrent.index_info.torrent_file.contents = torrent_with_19_pieces.to_vec();

                let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

                let response = client.upload_torrent(form.into()).await;

                assert_eq!(response.status, 400);
            }
        }

        #[tokio::test]
        async fn it_should_not_allow_uploading_a_torrent_with_a_non_existing_category() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            let uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let mut test_torrent = random_torrent();

            test_torrent.index_info.category = "non-existing-category".to_string();

            let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

            let response = client.upload_torrent(form.into()).await;

            assert_eq!(response.status, 400);
        }

        #[tokio::test]
        async fn it_should_not_allow_uploading_a_torrent_with_a_title_that_already_exists() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

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

            assert_json_error_response(&response, "This torrent title has already been used.");
        }

        #[tokio::test]
        async fn it_should_not_allow_uploading_a_torrent_with_a_info_hash_that_already_exists() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            // Upload the first torrent
            let first_torrent = random_torrent();
            let mut first_torrent_clone = first_torrent.clone();
            let first_torrent_title = first_torrent.index_info.title.clone();
            let form: UploadTorrentMultipartForm = first_torrent.index_info.into();
            let _response = client.upload_torrent(form.into()).await;

            // Upload the second torrent with the same info-hash as the first one.
            // We need to change the title otherwise the torrent will be rejected
            // because of the duplicate title.
            first_torrent_clone.index_info.title = format!("{first_torrent_title}-clone");
            let form: UploadTorrentMultipartForm = first_torrent_clone.index_info.into();
            let response = client.upload_torrent(form.into()).await;

            assert_eq!(response.status, 400);
        }

        #[tokio::test]
        async fn it_should_not_allow_uploading_a_torrent_whose_canonical_info_hash_already_exists() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let id1 = Uuid::new_v4();

            // Upload the first torrent
            let first_torrent = TestTorrent::with_custom_info_dict_field(id1, "data", "custom 01");
            let first_torrent_title = first_torrent.index_info.title.clone();
            let form: UploadTorrentMultipartForm = first_torrent.index_info.into();
            let _response = client.upload_torrent(form.into()).await;

            // Upload the second torrent with the same canonical info-hash as the first one.
            // We need to change the title otherwise the torrent will be rejected
            // because of the duplicate title.
            let mut torrent_with_the_same_canonical_info_hash =
                TestTorrent::with_custom_info_dict_field(id1, "data", "custom 02");
            torrent_with_the_same_canonical_info_hash.index_info.title = format!("{first_torrent_title}-clone");
            let form: UploadTorrentMultipartForm = torrent_with_the_same_canonical_info_hash.index_info.into();
            let response = client.upload_torrent(form.into()).await;

            assert_eq!(response.status, 400);
        }
    }

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_download_a_torrent_with_a_personal_announce_url() {
        let mut env = TestEnv::new();
        env.start(api::Version::V1).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        // Given a previously uploaded torrent
        let uploader = new_logged_in_user(&env).await;
        let (test_torrent, _torrent_listed_in_index) = upload_random_torrent_to_index(&uploader, &env).await;

        // And a logged in user who is going to download the torrent
        let downloader = new_logged_in_user(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &downloader.token);

        // When the user downloads the torrent
        let response = client.download_torrent(&test_torrent.file_info_hash()).await;

        let torrent = decode_torrent(&response.bytes).expect("could not decode downloaded torrent");

        // Then the torrent should have the personal announce URL
        let tracker_key = get_user_tracker_key(&downloader, &env)
            .await
            .expect("uploader should have a valid tracker key");

        let tracker_url = env.server_settings().unwrap().tracker.url;

        assert_eq!(
            torrent.announce.unwrap(),
            build_announce_url(&tracker_url, &Some(tracker_key))
        );
    }

    mod and_non_admins {

        use torrust_index_backend::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

        #[tokio::test]
        async fn it_should_not_allow_non_admins_to_delete_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let response = client.delete_torrent(&test_torrent.file_info_hash()).await;

            assert_eq!(response.status, 403);
        }

        #[tokio::test]
        async fn it_should_not_allow_non_admin_users_to_update_someone_elses_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            // Given a users uploads a torrent
            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            // Then another non admin user should not be able to update the torrent
            let not_the_uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &not_the_uploader.token);

            let new_title = format!("{}-new-title", test_torrent.index_info.title);
            let new_description = format!("{}-new-description", test_torrent.index_info.description);

            let response = client
                .update_torrent(
                    &test_torrent.file_info_hash(),
                    UpdateTorrentFrom {
                        title: Some(new_title.clone()),
                        description: Some(new_description.clone()),
                        category: None,
                        tags: None,
                    },
                )
                .await;

            assert_eq!(response.status, 403);
        }
    }

    mod and_torrent_owners {

        use torrust_index_backend::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
        use crate::common::contexts::torrent::responses::UpdatedTorrentResponse;
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

        #[tokio::test]
        async fn it_should_allow_torrent_owners_to_update_their_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let new_title = format!("{}-new-title", test_torrent.index_info.title);
            let new_description = format!("{}-new-description", test_torrent.index_info.description);

            let response = client
                .update_torrent(
                    &test_torrent.file_info_hash(),
                    UpdateTorrentFrom {
                        title: Some(new_title.clone()),
                        description: Some(new_description.clone()),
                        category: None,
                        tags: None,
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

        use torrust_index_backend::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
        use crate::common::contexts::torrent::responses::{DeletedTorrentResponse, UpdatedTorrentResponse};
        use crate::e2e::environment::TestEnv;
        use crate::e2e::web::api::v1::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::web::api::v1::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};

        #[tokio::test]
        async fn it_should_allow_admins_to_delete_torrents_searching_by_info_hash() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let admin = new_logged_in_admin(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &admin.token);

            let response = client.delete_torrent(&test_torrent.file_info_hash()).await;

            let deleted_torrent_response: DeletedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            assert_eq!(deleted_torrent_response.data.torrent_id, uploaded_torrent.torrent_id);
            assert!(response.is_json_and_ok());
        }

        #[tokio::test]
        async fn it_should_allow_admins_to_update_someone_elses_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Version::V1).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let logged_in_admin = new_logged_in_admin(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &logged_in_admin.token);

            let new_title = format!("{}-new-title", test_torrent.index_info.title);
            let new_description = format!("{}-new-description", test_torrent.index_info.description);

            let response = client
                .update_torrent(
                    &test_torrent.file_info_hash(),
                    UpdateTorrentFrom {
                        title: Some(new_title.clone()),
                        description: Some(new_description.clone()),
                        category: None,
                        tags: None,
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
