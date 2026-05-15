use std::process::ExitCode;

use torrust_index::app;
use torrust_index::bootstrap::config::initialize_configuration;
use torrust_index::web::api::Version;
use torrust_index_cli_common::{CommandExit, install_json_panic_hook};
use tracing::error;

const COMMAND_NAME: &str = "torrust-index";

#[tokio::main]
async fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    let configuration = initialize_configuration();

    let api_version = Version::V1;

    let app = app::run(configuration, &api_version).await;

    if app.api_server_halt_task.is_closed() {
        error!("halt channel closed before server shutdown");
        return CommandExit::Failure.exit_code();
    }

    match api_version {
        Version::V1 => match app.api_server.await {
            Ok(Ok(())) => CommandExit::Success.exit_code(),
            Ok(Err(error)) => {
                error!(%error, "API server failed");
                CommandExit::Failure.exit_code()
            }
            Err(error) => {
                error!(%error, "API server task was dropped");
                CommandExit::Failure.exit_code()
            }
        },
    }
}
