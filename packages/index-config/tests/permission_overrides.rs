//! Integration test: `[[permissions.overrides]]` is parsed and
//! preserved through the loader.
//!
//! ## Index
//!
//! | Test                                          | What it proves                                            |
//! |-----------------------------------------------|-----------------------------------------------------------|
//! | `multiple_overrides_load_in_declared_order`   | Order is stable; each tuple decodes correctly.            |
//! | `unknown_action_is_a_config_error`            | A typo in `action` surfaces as `Error::ConfigError`.      |
//! | `effect_must_be_lowercase`                    | `"Allow"` (capitalised) fails to deserialise.             |

use torrust_index_config::permissions::{Action, Effect, Role};
use torrust_index_config::test_helpers::PLACEHOLDER_TOML as BASE;
use torrust_index_config::{Error, Info, load_settings};

#[test]
fn multiple_overrides_load_in_declared_order() {
    let toml = format!(
        "{BASE}\n\
         [[permissions.overrides]]\n\
         role = \"registered\"\n\
         action = \"DeleteTorrent\"\n\
         effect = \"allow\"\n\
         \n\
         [[permissions.overrides]]\n\
         role = \"moderator\"\n\
         action = \"BanUser\"\n\
         effect = \"deny\"\n"
    );

    let settings = load_settings(&Info::from_toml(&toml)).expect("permissions overrides must load");
    let overrides = &settings.permissions.overrides;
    assert_eq!(overrides.len(), 2);
    assert_eq!(overrides[0].role, Role::Registered);
    assert_eq!(overrides[0].action, Action::DeleteTorrent);
    assert_eq!(overrides[0].effect, Effect::Allow);
    assert_eq!(overrides[1].role, Role::Moderator);
    assert_eq!(overrides[1].action, Action::BanUser);
    assert_eq!(overrides[1].effect, Effect::Deny);
}

#[test]
fn unknown_action_is_a_config_error() {
    let toml = format!(
        "{BASE}\n\
         [[permissions.overrides]]\n\
         role = \"registered\"\n\
         action = \"OverthrowTheState\"\n\
         effect = \"allow\"\n"
    );
    match load_settings(&Info::from_toml(&toml)) {
        Err(Error::ConfigError { source }) => {
            let msg = source.to_string();
            assert!(
                msg.contains("OverthrowTheState") || msg.contains("variant"),
                "unexpected: {msg}"
            );
        }
        other => panic!("expected ConfigError, got {other:?}"),
    }
}

#[test]
fn effect_must_be_lowercase() {
    let toml = format!(
        "{BASE}\n\
         [[permissions.overrides]]\n\
         role = \"registered\"\n\
         action = \"DeleteTorrent\"\n\
         effect = \"Allow\"\n"
    );
    assert!(matches!(
        load_settings(&Info::from_toml(&toml)),
        Err(Error::ConfigError { .. })
    ));
}
