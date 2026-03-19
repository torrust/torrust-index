#[test]
#[allow(clippy::result_large_err)]
fn it_should_load_with_default_config() {
    use crate::bootstrap::config::initialize_configuration;

    figment::Jail::expect_with(|jail| {
        let config_toml = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/share/default/config/index.development.sqlite3.toml"
        ));
        jail.set_env("TORRUST_INDEX_CONFIG_TOML", config_toml);

        drop(initialize_configuration());
        Ok(())
    });
}
