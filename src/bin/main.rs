use torrust_index_backend::app;
use torrust_index_backend::bootstrap::config::init_configuration;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = init_configuration().await;

    let app = app::run(configuration).await;

    app.api_server.await
}
