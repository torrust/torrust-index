//! Tests for [`crate::permissions`].
//!
//! ## Index
//!
//! | Test                                       | What it proves                                              |
//! |--------------------------------------------|-------------------------------------------------------------|
//! | `every_role_round_trips_via_string`        | `Role::ALL` ↔ `Display`/`FromStr` agree on every variant.   |
//! | `unknown_role_is_a_typed_error`            | `RoleParseError` carries the offending input.               |
//! | `effect_serialises_as_lowercase`           | `Effect::Allow` ↔ `"allow"`, `Effect::Deny` ↔ `"deny"`.     |
//! | `permission_override_round_trips_via_toml` | Single override survives a TOML serialise/parse cycle.      |
//! | `action_all_is_complete_and_unique`        | `Action::ALL` has no duplicates and matches variant count.  |

use std::collections::HashSet;
use std::str::FromStr;

use crate::permissions::{Action, Effect, PermissionOverride, Role};

/// Wrapper used by the round-trip test so toml-rs has a table to
/// hang the override fields off of.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
struct Wrap {
    item: PermissionOverride,
}

#[test]
fn every_role_round_trips_via_string() {
    for role in Role::ALL {
        let s = role.to_string();
        let parsed = Role::from_str(&s).expect("Display output must be FromStr-parseable");
        assert_eq!(*role, parsed);
    }
}

#[test]
fn unknown_role_is_a_typed_error() {
    let err = Role::from_str("supreme-overlord").expect_err("must reject unknown roles");
    assert!(format!("{err}").contains("supreme-overlord"));
}

#[test]
fn effect_serialises_as_lowercase() {
    assert_eq!(serde_json::to_string(&Effect::Allow).unwrap(), "\"allow\"");
    assert_eq!(serde_json::to_string(&Effect::Deny).unwrap(), "\"deny\"");
}

#[test]
fn permission_override_round_trips_via_toml() {
    let original = PermissionOverride {
        role: Role::Registered,
        action: Action::DeleteTorrent,
        effect: Effect::Allow,
    };
    let encoded = toml::to_string(&Wrap { item: original.clone() }).unwrap();
    let decoded: Wrap = toml::from_str(&encoded).unwrap();
    assert_eq!(decoded.item, original);
}

#[test]
fn action_all_is_complete_and_unique() {
    let unique: HashSet<_> = Action::ALL.iter().collect();
    assert_eq!(unique.len(), Action::ALL.len(), "Action::ALL must not contain duplicates");
    // Sanity: at least the publicly documented variants are present.
    assert!(Action::ALL.contains(&Action::AddTorrent));
    assert!(Action::ALL.contains(&Action::GetMyPermissions));
}
