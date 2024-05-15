use torrust_index::app;
use torrust_index::bootstrap::config::initialize_configuration;
use torrust_index::web::api::Version;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = initialize_configuration();

    let api_version = Version::V1;

    let app = app::run(configuration, &api_version).await;

    assert!(!app.api_server_halt_task.is_closed(), "Halt channel should be open");

    match api_version {
        Version::V1 => app.api_server.await.expect("the API server was dropped"),
    }
}
