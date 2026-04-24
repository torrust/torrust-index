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

        // Per ADR-T-009 §D2, the shipped dev sample no longer carries
        // `tracker.token` or `database.connect_url` — the operator
        // supplies them at runtime via env-var overrides. Mirror that
        // workflow here so bootstrap can resolve the mandatory fields.
        jail.set_env("TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN", "MyAccessToken");
        jail.set_env(
            "TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL",
            "sqlite://data.db?mode=rwc",
        );

        drop(initialize_configuration());
        Ok(())
    });
}
