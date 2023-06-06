use torrust_index_backend::app;
use torrust_index_backend::bootstrap::config::init_configuration;
use torrust_index_backend::web::api::Implementation;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = init_configuration().await;

    // todo: we are migrating from actix-web to axum, so we need to keep both
    // implementations for a while. For production we only use ActixWeb.
    // Once the Axum implementation is finished and stable, we can switch to it
    // and remove the ActixWeb implementation.
    let api_implementation = Implementation::ActixWeb;

    let app = app::run(configuration, &api_implementation).await;

    match api_implementation {
        Implementation::ActixWeb => app.actix_web_api_server.unwrap().await.expect("the API server was dropped"),
        Implementation::Axum => app.axum_api_server.unwrap().await.expect("the Axum API server was dropped"),
    }
}
