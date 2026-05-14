//! # ADR-T-010 CLI contract regression tests
//!
//! | Test                                             | What it covers                                      |
//! |--------------------------------------------------|-----------------------------------------------------|
//! | `in_scope_binaries_return_exit_code_from_main`   | In-scope binaries keep explicit `ExitCode` mains.   |

const IN_SCOPE_BINARIES: &[(&str, &str)] = &[
    ("src/main.rs", include_str!("../src/main.rs")),
    (
        "src/bin/create_test_torrent.rs",
        include_str!("../src/bin/create_test_torrent.rs"),
    ),
    (
        "src/bin/import_tracker_statistics.rs",
        include_str!("../src/bin/import_tracker_statistics.rs"),
    ),
    ("src/bin/parse_torrent.rs", include_str!("../src/bin/parse_torrent.rs")),
    ("src/bin/seeder.rs", include_str!("../src/bin/seeder.rs")),
    ("src/bin/upgrade.rs", include_str!("../src/bin/upgrade.rs")),
    (
        "packages/index-auth-keypair/src/bin/torrust-index-auth-keypair.rs",
        include_str!("../packages/index-auth-keypair/src/bin/torrust-index-auth-keypair.rs"),
    ),
    (
        "packages/index-config-probe/src/bin/torrust-index-config-probe.rs",
        include_str!("../packages/index-config-probe/src/bin/torrust-index-config-probe.rs"),
    ),
    (
        "packages/index-health-check/src/bin/torrust-index-health-check.rs",
        include_str!("../packages/index-health-check/src/bin/torrust-index-health-check.rs"),
    ),
];

#[test]
fn in_scope_binaries_return_exit_code_from_main() {
    let mut failures = Vec::new();

    for &(relative_path, source) in IN_SCOPE_BINARIES {
        let Some(signature) = main_signature(source) else {
            failures.push(format!("{relative_path}: missing main function"));
            continue;
        };

        if !signature.contains("-> ExitCode") {
            failures.push(format!(
                "{relative_path}: main must return ExitCode instead of using Rust's default termination: `{signature}`"
            ));
        }

        if signature.contains("-> Result") {
            failures.push(format!(
                "{relative_path}: main must not return Result because default termination writes raw stderr: `{signature}`"
            ));
        }
    }

    let message = failures.join("\n");
    assert!(failures.is_empty(), "{message}");
}

fn main_signature(source: &str) -> Option<String> {
    let start = source.find("fn main(")?;
    let rest = &source[start..];
    let end = rest.find('{').unwrap_or(rest.len());

    Some(collapse_whitespace(&rest[..end]))
}

fn collapse_whitespace(value: &str) -> String {
    let mut collapsed = String::new();

    for part in value.split_whitespace() {
        if !collapsed.is_empty() {
            collapsed.push(' ');
        }
        collapsed.push_str(part);
    }

    collapsed
}
