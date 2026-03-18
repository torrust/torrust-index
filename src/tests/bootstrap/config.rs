#[test]
fn it_should_load_with_default_config() {
    use crate::bootstrap::config::initialize_configuration;

    drop(initialize_configuration());
}
