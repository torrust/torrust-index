mod v2;

use url::Url;

use crate::config::{ApiToken, Configuration, Info, Settings};

/// Remove `TORRUST_INDEX_CONFIG_*` env vars that may have been
/// inherited from the surrounding shell (e.g. the e2e runner scripts
/// export `TORRUST_INDEX_CONFIG_OVERRIDE_DATABASE__CONNECT_URL`
/// before invoking `cargo test`). Tests that assert on the *default*
/// configuration must scrub these before loading settings; otherwise
/// figment's `Env` provider merges the ambient overrides on top of
/// the fixture and the equality assertions diverge.
///
/// Safe to call from inside a `figment::Jail` closure: the jail holds
/// a global mutex while running and snapshots/restores the process
/// environment around the closure.
fn clear_inherited_config_env() {
    let names: Vec<String> = std::env::vars()
        .map(|(k, _)| k)
        .filter(|k| {
            k == "TORRUST_INDEX_CONFIG_TOML"
                || k == "TORRUST_INDEX_CONFIG_TOML_PATH"
                || k.starts_with("TORRUST_INDEX_CONFIG_OVERRIDE_")
        })
        .collect();
    for name in names {
        // SAFETY: Edition 2024 marks `remove_var` unsafe because env
        // mutation is not thread-safe. We only call this from inside
        // a `figment::Jail` closure, which serialises tests via a
        // global mutex and restores the snapshot on drop.
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var(name);
        }
    }
}

fn default_config_toml() -> String {
    use std::fs;
    use std::path::PathBuf;

    // Get the path to the current Cargo.toml directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR environment variable not set");

    // Construct the path to the default configuration file relative to the Cargo.toml directory
    let mut path = PathBuf::from(manifest_dir);
    path.push("tests/fixtures/default_configuration.toml");

    let config = fs::read_to_string(path)
        .expect("Could not read default configuration TOML file: tests/fixtures/default_configuration.toml");

    config.lines().map(str::trim_start).collect::<Vec<&str>>().join("\n")
}

/// Build settings from default configuration fixture in TOML.
///
/// We just want to load that file without overriding with env var or other
/// configuration loading behavior.
fn default_settings() -> Settings {
    use figment::Figment;
    use figment::providers::{Format, Toml};

    let figment = Figment::from(Toml::string(&default_config_toml()));
    let settings: Settings = figment.extract().expect("Invalid configuration");

    settings
}

#[test]
#[allow(clippy::result_large_err)]
fn configuration_should_have_a_test_constructor() {
    // Wrapped in `figment::Jail` so it serialises (via figment's
    // global mutex) with sibling tests that mutate
    // `TORRUST_INDEX_CONFIG_OVERRIDE_*` env vars. Without the jail,
    // those overrides leak across parallel tests and `for_tests()`
    // reads the leaked token instead of the PLACEHOLDER_TOML value.
    //
    // The test is intentionally `#[test]` (not `#[tokio::test]`) so
    // we can call `RwLock::blocking_read` inside the synchronous
    // Jail closure without panicking under a tokio runtime.
    figment::Jail::expect_with(|_| {
        clear_inherited_config_env();

        let settings = Configuration::for_tests().settings.blocking_read().clone();

        // Mandatory fields from PLACEHOLDER_TOML are present.
        assert_eq!(settings.tracker.token.to_string(), "MyAccessToken");
        assert_eq!(settings.database.connect_url.as_str(), "sqlite://data.db?mode=rwc");

        Ok(())
    });
}

#[tokio::test]
async fn configuration_should_return_the_site_name() {
    let configuration = Configuration::for_tests();
    assert_eq!(configuration.get_site_name().await, "Torrust".to_string());
}

#[tokio::test]
async fn configuration_should_return_the_api_base_url() {
    let configuration = Configuration::for_tests();
    assert_eq!(configuration.get_api_base_url().await, None);

    let mut settings_lock = configuration.settings.write().await;
    settings_lock.net.base_url = Some(Url::parse("http://localhost").unwrap());
    drop(settings_lock);

    assert_eq!(configuration.get_api_base_url().await, Some("http://localhost/".to_string()));
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn configuration_could_be_loaded_from_a_toml_string() {
    figment::Jail::expect_with(|jail| {
        clear_inherited_config_env();

        jail.create_dir("templates")?;
        jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

        let info = Info {
            config_toml: Some(default_config_toml()),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Failed to load configuration from info");

        assert_eq!(settings, default_settings());

        Ok(())
    });
}

#[test]
#[allow(clippy::result_large_err)]
fn configuration_should_use_the_default_values_when_only_the_mandatory_options_are_provided_by_the_user_via_toml_file() {
    figment::Jail::expect_with(|jail| {
        clear_inherited_config_env();

        jail.create_file(
            "index.toml",
            r#"
                [metadata]
                schema_version = "2.0.0"

                [logging]
                threshold = "info"

                [tracker]
                token = "MyAccessToken"

                [database]
                connect_url = "sqlite://data.db?mode=rwc"

                [auth]
            "#,
        )?;

        let info = Info {
            config_toml: None,
            config_toml_path: "index.toml".to_string(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings, default_settings());

        Ok(())
    });
}

#[test]
#[allow(clippy::result_large_err)]
fn configuration_should_use_the_default_values_when_only_the_mandatory_options_are_provided_by_the_user_via_toml_content() {
    figment::Jail::expect_with(|_jail| {
        clear_inherited_config_env();

        let config_toml = r#"
                [metadata]
                schema_version = "2.0.0"

                [logging]
                threshold = "info"

                [tracker]
                token = "MyAccessToken"

                [database]
                connect_url = "sqlite://data.db?mode=rwc"

                [auth]
            "#
        .to_string();

        let info = Info {
            config_toml: Some(config_toml),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings, default_settings());

        Ok(())
    });
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn configuration_should_allow_to_override_the_tracker_api_token_provided_in_the_toml_file() {
    figment::Jail::expect_with(|jail| {
        jail.create_dir("templates")?;
        jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

        jail.set_env("TORRUST_INDEX_CONFIG_OVERRIDE_TRACKER__TOKEN", "OVERRIDDEN API TOKEN");

        let info = Info {
            config_toml: Some(default_config_toml()),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings.tracker.token, ApiToken::new("OVERRIDDEN API TOKEN"));

        Ok(())
    });
}

#[tokio::test]
#[allow(clippy::result_large_err)]
async fn configuration_should_allow_to_override_the_private_key_path_provided_in_the_toml_file() {
    figment::Jail::expect_with(|jail| {
        jail.create_dir("templates")?;
        jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

        jail.set_env(
            "TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__PRIVATE_KEY_PATH",
            "/custom/path/private.pem",
        );

        let info = Info {
            config_toml: Some(default_config_toml()),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings.auth.private_key_path, Some("/custom/path/private.pem".to_owned()));

        Ok(())
    });
}

mod semantic_validation {
    use url::Url;

    use crate::config::Configuration;
    use crate::config::validator::Validator;

    #[tokio::test]
    async fn udp_trackers_in_private_mode_are_not_supported() {
        let configuration = Configuration::for_tests();

        let mut settings_lock = configuration.settings.write().await;
        settings_lock.tracker.private = true;
        settings_lock.tracker.url = Url::parse("udp://localhost:6969").unwrap();

        let validation_result = settings_lock.validate();
        drop(settings_lock);

        assert!(validation_result.is_err());
    }
}
