//! API contract for `proxy` context.

/* mod for_guest_users {
    use torrust_index::web::api;

    use crate::common::client::Client;
    use crate::common::contexts::torrent::fixtures::random_torrent;
    use crate::e2e::environment::TestEnv;
    use crate::e2e::web::api::v1::contexts::torrent::steps::upload_torrent;
    use crate::e2e::web::api::v1::contexts::user::steps::new_logged_in_user;

    #[tokio::test]
    async fn it_should_not_allow_guest_user_to_get_a_proxied_image_by_url() {
        let mut env = TestEnv::new();

        env.start(api::Version::V1).await;

        /* if !env.provides_a_tracker() {
            println!("test skipped. It requires a tracker to be running.");
            return;
        } */

        let mut test_torrent = random_torrent();

        test_torrent.index_info.description = String::from(
            "![image info](https://upload.wikimedia.org/wikipedia/commons/thumb/7/71/Zaadpluizen_van_een_Clematis_texensis_%27Princess_Diana%27._18-07-2023_%28actm.%29_02.jpg/1024px-Zaadpluizen_van_een_Clematis_texensis_%27Princess_Diana%27._18-07-2023_%28actm.%29_02.jpg)",
        );

        let torrent_uploader = new_logged_in_user(&env).await;

        upload_torrent(&torrent_uploader, &test_torrent.index_info, &env).await;

        let client = Client::unauthenticated(&env.server_socket_addr().unwrap());

        let url = "https%3A%2F%2Fupload.wikimedia.org%2Fwikipedia%2Fcommons%2Fthumb%2F7%2F71%2FZaadpluizen_van_een_Clematis_texensis_%2527Princess_Diana%2527._18-07-2023_%2528actm.%2529_02.jpg%2F1024px-Zaadpluizen_van_een_Clematis_texensis_%2527Princess_Diana%2527._18-07-2023_%2528actm.%2529_02.jpg";

        let response = client.get_image_by_url(url).await;

        assert_eq!(response.status, 401);
    }
} */
