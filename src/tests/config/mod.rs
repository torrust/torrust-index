mod v2;

use url::Url;

use crate::config::{ApiToken, Configuration, Info, SecretKey, Settings};

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

#[tokio::test]
async fn configuration_should_have_a_default_constructor() {
    let settings = Configuration::default().get_all().await;

    assert_eq!(settings, default_settings());
}

#[tokio::test]
async fn configuration_should_return_the_site_name() {
    let configuration = Configuration::default();
    assert_eq!(configuration.get_site_name().await, "Torrust".to_string());
}

#[tokio::test]
async fn configuration_should_return_the_api_base_url() {
    let configuration = Configuration::default();
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
        jail.create_dir("templates")?;
        jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

        let info = Info {
            config_toml: Some(default_config_toml()),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Failed to load configuration from info");

        assert_eq!(settings, Settings::default());

        Ok(())
    });
}

#[test]
#[allow(clippy::result_large_err)]
fn configuration_should_use_the_default_values_when_only_the_mandatory_options_are_provided_by_the_user_via_toml_file() {
    figment::Jail::expect_with(|jail| {
        jail.create_file(
            "index.toml",
            r#"
                [metadata]
                schema_version = "2.0.0"

                [logging]
                threshold = "info"

                [tracker]
                token = "MyAccessToken"

                [auth]
                jwt_signing_secret = "MaxVerstappenWC2021"
            "#,
        )?;

        let info = Info {
            config_toml: None,
            config_toml_path: "index.toml".to_string(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings, Settings::default());

        Ok(())
    });
}

#[test]
#[allow(clippy::result_large_err)]
fn configuration_should_use_the_default_values_when_only_the_mandatory_options_are_provided_by_the_user_via_toml_content() {
    figment::Jail::expect_with(|_jail| {
        let config_toml = r#"
                [metadata]
                schema_version = "2.0.0"

                [logging]
                threshold = "info"

                [tracker]
                token = "MyAccessToken"

                [auth]
                jwt_signing_secret = "MaxVerstappenWC2021"
            "#
        .to_string();

        let info = Info {
            config_toml: Some(config_toml),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings, Settings::default());

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
async fn configuration_should_allow_to_override_the_authentication_jwt_signing_secret_provided_in_the_toml_file() {
    figment::Jail::expect_with(|jail| {
        jail.create_dir("templates")?;
        jail.create_file("templates/verify.html", "EMAIL TEMPLATE")?;

        jail.set_env(
            "TORRUST_INDEX_CONFIG_OVERRIDE_AUTH__JWT_SIGNING_SECRET",
            "OVERRIDDEN AUTH SECRET KEY",
        );

        let info = Info {
            config_toml: Some(default_config_toml()),
            config_toml_path: String::new(),
        };

        let settings = Configuration::load_settings(&info).expect("Could not load configuration from file");

        assert_eq!(settings.auth.jwt_signing_secret, SecretKey::new("OVERRIDDEN AUTH SECRET KEY"));

        Ok(())
    });
}

mod semantic_validation {
    use url::Url;

    use crate::config::Configuration;
    use crate::config::validator::Validator;

    #[tokio::test]
    async fn udp_trackers_in_private_mode_are_not_supported() {
        let configuration = Configuration::default();

        let mut settings_lock = configuration.settings.write().await;
        settings_lock.tracker.private = true;
        settings_lock.tracker.url = Url::parse("udp://localhost:6969").unwrap();

        let validation_result = settings_lock.validate();
        drop(settings_lock);

        assert!(validation_result.is_err());
    }
}
