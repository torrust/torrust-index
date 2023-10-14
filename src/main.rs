use torrust_index::app;
use torrust_index::bootstrap::config::initialize_configuration;
use torrust_index::web::api::Version;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = initialize_configuration();

    let api_version = Version::V1;

    let app = app::run(configuration, &api_version).await;

    match api_version {
        Version::V1 => app.api_server.unwrap().await.expect("the API server was dropped"),
    }
}
