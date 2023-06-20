use torrust_index_backend::app;
use torrust_index_backend::bootstrap::config::init_configuration;
use torrust_index_backend::web::api::Implementation;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = init_configuration().await;

    let api_implementation = Implementation::Axum;

    let app = app::run(configuration, &api_implementation).await;

    match api_implementation {
        Implementation::ActixWeb => app.actix_web_api_server.unwrap().await.expect("the API server was dropped"),
        Implementation::Axum => app.axum_api_server.unwrap().await.expect("the Axum API server was dropped"),
    }
}
