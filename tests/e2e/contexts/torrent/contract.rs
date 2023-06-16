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

    use crate::common::client::Client;
    use crate::common::contexts::category::fixtures::software_predefined_category_id;
    use crate::common::contexts::torrent::asserts::assert_expected_torrent_details;
    use crate::common::contexts::torrent::requests::InfoHash;
    use crate::common::contexts::torrent::responses::{
        Category, File, TorrentDetails, TorrentDetailsResponse, TorrentListResponse,
    };
    use crate::common::http::{Query, QueryParam};
    use crate::e2e::contexts::torrent::asserts::expected_torrent;
    use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
    use crate::e2e::contexts::user::steps::new_logged_in_user;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    async fn it_should_allow_guests_to_get_torrents() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

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
        env.start(api::Implementation::ActixWeb).await;

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
        env.start(api::Implementation::ActixWeb).await;

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
        env.start(api::Implementation::ActixWeb).await;

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
        env.start(api::Implementation::ActixWeb).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let uploader = new_logged_in_user(&env).await;
        let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

        let response = client.get_torrent(&test_torrent.info_hash()).await;

        let torrent_details_response: TorrentDetailsResponse = serde_json::from_str(&response.body).unwrap();

        let tracker_url = env.server_settings().unwrap().tracker.url;
        let encoded_tracker_url = urlencoding::encode(&tracker_url);

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
            // code-review: why is this duplicated? It seems that is adding the
            // same tracker twice because first ti adds all trackers and then
            // it adds the tracker with the personal announce url, if the user
            // is logged in. If the user is not logged in, it adds the default
            // tracker again, and it ends up with two trackers.
            trackers: vec![tracker_url.clone(), tracker_url.clone()],
            magnet_link: format!(
                // cspell:disable-next-line
                "magnet:?xt=urn:btih:{}&dn={}&tr={}&tr={}",
                test_torrent.file_info.info_hash.to_uppercase(),
                urlencoding::encode(&test_torrent.index_info.title),
                encoded_tracker_url,
                encoded_tracker_url
            ),
        };

        assert_expected_torrent_details(&torrent_details_response.data, &expected_torrent);
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_allow_guests_to_download_a_torrent_file_searching_by_info_hash() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let uploader = new_logged_in_user(&env).await;
        let (test_torrent, _torrent_listed_in_index) = upload_random_torrent_to_index(&uploader, &env).await;

        let response = client.download_torrent(&test_torrent.info_hash()).await;

        let torrent = decode_torrent(&response.bytes).expect("could not decode downloaded torrent");
        let uploaded_torrent =
            decode_torrent(&test_torrent.index_info.torrent_file.contents).expect("could not decode uploaded torrent");
        let expected_torrent = expected_torrent(uploaded_torrent, &env, &None).await;
        assert_eq!(torrent, expected_torrent);
        assert!(response.is_bittorrent_and_ok());
    }

    #[tokio::test]
    async fn it_should_return_a_not_found_trying_to_download_a_non_existing_torrent() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let non_existing_info_hash: InfoHash = "443c7602b4fde83d1154d6d9da48808418b181b6".to_string();

        let response = client.download_torrent(&non_existing_info_hash).await;

        // code-review: should this be 404?
        assert_eq!(response.status, 400);
    }

    #[tokio::test]
    async fn it_should_not_allow_guests_to_delete_torrents() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let uploader = new_logged_in_user(&env).await;
        let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

        let response = client.delete_torrent(&test_torrent.info_hash()).await;

        assert_eq!(response.status, 401);
    }
}

mod for_authenticated_users {

    use torrust_index_backend::utils::parse_torrent::decode_torrent;
    use torrust_index_backend::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::torrent::fixtures::random_torrent;
    use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
    use crate::common::contexts::torrent::responses::UploadedTorrentResponse;
    use crate::e2e::contexts::torrent::asserts::{build_announce_url, get_user_tracker_key};
    use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
    use crate::e2e::contexts::user::steps::new_logged_in_user;
    use crate::e2e::environment::TestEnv;

    #[tokio::test]
    async fn it_should_allow_authenticated_users_to_upload_new_torrents() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

        if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        }

        let uploader = new_logged_in_user(&env).await;
        let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

        let test_torrent = random_torrent();
        let info_hash = test_torrent.info_hash().clone();

        let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

        let response = client.upload_torrent(form.into()).await;

        let uploaded_torrent_response: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();

        assert_eq!(
            uploaded_torrent_response.data.info_hash.to_lowercase(),
            info_hash.to_lowercase()
        );
        assert!(response.is_json_and_ok());
    }

    #[tokio::test]
    async fn it_should_not_allow_uploading_a_torrent_with_a_non_existing_category() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

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
        env.start(api::Implementation::ActixWeb).await;

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

        assert_eq!(response.body, "{\"error\":\"This torrent title has already been used.\"}");
        assert_eq!(response.status, 400);
    }

    #[tokio::test]
    async fn it_should_not_allow_uploading_a_torrent_with_a_info_hash_that_already_exists() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

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
    async fn it_should_allow_authenticated_users_to_download_a_torrent_with_a_personal_announce_url() {
        let mut env = TestEnv::new();
        env.start(api::Implementation::ActixWeb).await;

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
        let response = client.download_torrent(&test_torrent.info_hash()).await;

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
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::new_logged_in_user;
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        async fn it_should_not_allow_non_admins_to_delete_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::ActixWeb).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let response = client.delete_torrent(&test_torrent.info_hash()).await;

            assert_eq!(response.status, 403);
        }

        #[tokio::test]
        async fn it_should_allow_non_admin_users_to_update_someone_elses_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::ActixWeb).await;

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
                    &test_torrent.info_hash(),
                    UpdateTorrentFrom {
                        title: Some(new_title.clone()),
                        description: Some(new_description.clone()),
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
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::new_logged_in_user;
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        async fn it_should_allow_torrent_owners_to_update_their_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::ActixWeb).await;

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
                    &test_torrent.info_hash(),
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
        use torrust_index_backend::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
        use crate::common::contexts::torrent::responses::{DeletedTorrentResponse, UpdatedTorrentResponse};
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        async fn it_should_allow_admins_to_delete_torrents_searching_by_info_hash() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::ActixWeb).await;

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let admin = new_logged_in_admin(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &admin.token);

            let response = client.delete_torrent(&test_torrent.info_hash()).await;

            let deleted_torrent_response: DeletedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            assert_eq!(deleted_torrent_response.data.torrent_id, uploaded_torrent.torrent_id);
            assert!(response.is_json_and_ok());
        }

        #[tokio::test]
        async fn it_should_allow_admins_to_update_someone_elses_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::ActixWeb).await;

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
                    &test_torrent.info_hash(),
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

mod with_axum_implementation {

    mod for_guests {

        use std::env;

        use torrust_index_backend::utils::parse_torrent::decode_torrent;
        use torrust_index_backend::web::api;

        use crate::common::client::Client;
        use crate::common::contexts::category::fixtures::software_predefined_category_id;
        use crate::common::contexts::torrent::asserts::assert_expected_torrent_details;
        use crate::common::contexts::torrent::requests::InfoHash;
        use crate::common::contexts::torrent::responses::{
            Category, File, TorrentDetails, TorrentDetailsResponse, TorrentListResponse,
        };
        use crate::common::http::{Query, QueryParam};
        use crate::e2e::config::ENV_VAR_E2E_EXCLUDE_AXUM_IMPL;
        use crate::e2e::contexts::torrent::asserts::expected_torrent;
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::new_logged_in_user;
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        async fn it_should_allow_guests_to_get_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let response = client.get_torrent(&test_torrent.info_hash()).await;

            let torrent_details_response: TorrentDetailsResponse = serde_json::from_str(&response.body).unwrap();

            let tracker_url = env.server_settings().unwrap().tracker.url;
            let encoded_tracker_url = urlencoding::encode(&tracker_url);

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
                // code-review: why is this duplicated? It seems that is adding the
                // same tracker twice because first ti adds all trackers and then
                // it adds the tracker with the personal announce url, if the user
                // is logged in. If the user is not logged in, it adds the default
                // tracker again, and it ends up with two trackers.
                trackers: vec![tracker_url.clone(), tracker_url.clone()],
                magnet_link: format!(
                    // cspell:disable-next-line
                    "magnet:?xt=urn:btih:{}&dn={}&tr={}&tr={}",
                    test_torrent.file_info.info_hash.to_uppercase(),
                    urlencoding::encode(&test_torrent.index_info.title),
                    encoded_tracker_url,
                    encoded_tracker_url
                ),
            };

            assert_expected_torrent_details(&torrent_details_response.data, &expected_torrent);
            assert!(response.is_json_and_ok());
        }

        #[tokio::test]
        async fn it_should_allow_guests_to_download_a_torrent_file_searching_by_info_hash() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _torrent_listed_in_index) = upload_random_torrent_to_index(&uploader, &env).await;

            let response = client.download_torrent(&test_torrent.info_hash()).await;

            let torrent = decode_torrent(&response.bytes).expect("could not decode downloaded torrent");
            let uploaded_torrent =
                decode_torrent(&test_torrent.index_info.torrent_file.contents).expect("could not decode uploaded torrent");
            let expected_torrent = expected_torrent(uploaded_torrent, &env, &None).await;
            assert_eq!(torrent, expected_torrent);
            assert!(response.is_bittorrent_and_ok());
        }

        #[tokio::test]
        async fn it_should_return_a_not_found_trying_to_download_a_non_existing_torrent() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

            let non_existing_info_hash: InfoHash = "443c7602b4fde83d1154d6d9da48808418b181b6".to_string();

            let response = client.download_torrent(&non_existing_info_hash).await;

            // code-review: should this be 404?
            assert_eq!(response.status, 400);
        }

        /*

        #[tokio::test]
        async fn it_should_not_allow_guests_to_delete_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

            let uploader = new_logged_in_user(&env).await;
            let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

            let response = client.delete_torrent(&test_torrent.info_hash()).await;

            assert_eq!(response.status, 401);
        }

        */
    }

    mod for_authenticated_users {

        use std::env;

        use torrust_index_backend::utils::parse_torrent::decode_torrent;
        use torrust_index_backend::web::api;

        use crate::common::asserts::assert_json_error_response;
        use crate::common::client::Client;
        use crate::common::contexts::torrent::fixtures::random_torrent;
        use crate::common::contexts::torrent::forms::UploadTorrentMultipartForm;
        use crate::common::contexts::torrent::responses::UploadedTorrentResponse;
        use crate::e2e::config::ENV_VAR_E2E_EXCLUDE_AXUM_IMPL;
        use crate::e2e::contexts::torrent::asserts::{build_announce_url, get_user_tracker_key};
        use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
        use crate::e2e::contexts::user::steps::new_logged_in_user;
        use crate::e2e::environment::TestEnv;

        #[tokio::test]
        async fn it_should_allow_authenticated_users_to_upload_new_torrents() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

            if !env.provides_a_tracker() {
                println!("test skipped. It requires a tracker to be running.");
                return;
            }

            let uploader = new_logged_in_user(&env).await;
            let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

            let test_torrent = random_torrent();
            let info_hash = test_torrent.info_hash().clone();

            let form: UploadTorrentMultipartForm = test_torrent.index_info.into();

            let response = client.upload_torrent(form.into()).await;

            let uploaded_torrent_response: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();

            assert_eq!(
                uploaded_torrent_response.data.info_hash.to_lowercase(),
                info_hash.to_lowercase()
            );
            assert!(response.is_json_and_ok());
        }

        #[tokio::test]
        async fn it_should_not_allow_uploading_a_torrent_with_a_non_existing_category() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
        async fn it_should_allow_authenticated_users_to_download_a_torrent_with_a_personal_announce_url() {
            let mut env = TestEnv::new();
            env.start(api::Implementation::Axum).await;

            if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                println!("Skipped");
                return;
            }

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
            let response = client.download_torrent(&test_torrent.info_hash()).await;

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
            /*

            use std::env;

            use torrust_index_backend::web::api;

            use crate::common::client::Client;
            use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
            use crate::e2e::config::ENV_VAR_E2E_EXCLUDE_AXUM_IMPL;
            use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
            use crate::e2e::contexts::user::steps::new_logged_in_user;
            use crate::e2e::environment::TestEnv;

            #[tokio::test]
            async fn it_should_not_allow_non_admins_to_delete_torrents() {
                let mut env = TestEnv::new();
                env.start(api::Implementation::Axum).await;

                if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                    println!("Skipped");
                    return;
                }

                if !env.provides_a_tracker() {
                    println!("test skipped. It requires a tracker to be running.");
                    return;
                }

                let uploader = new_logged_in_user(&env).await;
                let (test_torrent, _uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &uploader.token);

                let response = client.delete_torrent(&test_torrent.info_hash()).await;

                assert_eq!(response.status, 403);
            }

            #[tokio::test]
            async fn it_should_allow_non_admin_users_to_update_someone_elses_torrents() {
                let mut env = TestEnv::new();
                env.start(api::Implementation::Axum).await;

                if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                    println!("Skipped");
                    return;
                }

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
                        &test_torrent.info_hash(),
                        UpdateTorrentFrom {
                            title: Some(new_title.clone()),
                            description: Some(new_description.clone()),
                        },
                    )
                    .await;

                assert_eq!(response.status, 403);
            }

            */
        }

        mod and_torrent_owners {
            /*

            use std::env;

            use torrust_index_backend::web::api;

            use crate::common::client::Client;
            use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
            use crate::common::contexts::torrent::responses::UpdatedTorrentResponse;
            use crate::e2e::config::ENV_VAR_E2E_EXCLUDE_AXUM_IMPL;
            use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
            use crate::e2e::contexts::user::steps::new_logged_in_user;
            use crate::e2e::environment::TestEnv;

            #[tokio::test]
            async fn it_should_allow_torrent_owners_to_update_their_torrents() {
                let mut env = TestEnv::new();
                env.start(api::Implementation::Axum).await;

                if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                    println!("Skipped");
                    return;
                }

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
                        &test_torrent.info_hash(),
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

            */
        }

        mod and_admins {
            /*

            use std::env;

            use torrust_index_backend::web::api;

            use crate::common::client::Client;
            use crate::common::contexts::torrent::forms::UpdateTorrentFrom;
            use crate::common::contexts::torrent::responses::{DeletedTorrentResponse, UpdatedTorrentResponse};
            use crate::e2e::config::ENV_VAR_E2E_EXCLUDE_AXUM_IMPL;
            use crate::e2e::contexts::torrent::steps::upload_random_torrent_to_index;
            use crate::e2e::contexts::user::steps::{new_logged_in_admin, new_logged_in_user};
            use crate::e2e::environment::TestEnv;

            #[tokio::test]
            async fn it_should_allow_admins_to_delete_torrents_searching_by_info_hash() {
                let mut env = TestEnv::new();
                env.start(api::Implementation::Axum).await;

                if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                    println!("Skipped");
                    return;
                }

                if !env.provides_a_tracker() {
                    println!("test skipped. It requires a tracker to be running.");
                    return;
                }

                let uploader = new_logged_in_user(&env).await;
                let (test_torrent, uploaded_torrent) = upload_random_torrent_to_index(&uploader, &env).await;

                let admin = new_logged_in_admin(&env).await;
                let client = Client::authenticated(&env.server_socket_addr().unwrap(), &admin.token);

                let response = client.delete_torrent(&test_torrent.info_hash()).await;

                let deleted_torrent_response: DeletedTorrentResponse = serde_json::from_str(&response.body).unwrap();

                assert_eq!(deleted_torrent_response.data.torrent_id, uploaded_torrent.torrent_id);
                assert!(response.is_json_and_ok());
            }

            #[tokio::test]
            async fn it_should_allow_admins_to_update_someone_elses_torrents() {
                let mut env = TestEnv::new();
                env.start(api::Implementation::Axum).await;

                if env::var(ENV_VAR_E2E_EXCLUDE_AXUM_IMPL).is_ok() {
                    println!("Skipped");
                    return;
                }

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
                        &test_torrent.info_hash(),
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

            */
        }
    }
}
