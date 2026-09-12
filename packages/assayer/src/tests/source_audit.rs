// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`no_stub_functions_in_codebase`] | audit | No placeholder function survives anywhere in production source: the whole source tree, minus test modules and the test subtree, contains no stub-prefixed definition. Scaffolding written to make a module compile before its algorithm existed is the kind of thing that quietly outlives the work it was standing in for, and a returned placeholder is far harder to notice than a missing function. Test code may still name stubs, because a test about a stub is not a stub. |
//! | [`stubs_module_deleted`] | audit | The module that once held the crate's placeholders is absent from the source tree and stays absent. A deletion is not self-enforcing: a merge, a revert, or a well-meaning restoration could bring the file back, and its mere existence would give new placeholder code an obvious home to be added to. |
//! | [`resonance_module_does_not_import_core_state`] | audit | The derivation layer's purity is a structural property of the source tree, not a convention: every crate-internal import under the resonance modules names a root on a closed allowlist, and any other root fails the build. Purity is what makes derivation reproducible from its arguments alone, and the visibility system cannot express a rule of this shape — a module is either visible to the crate or not — so the boundary is checked by reading the source. The list fails closed: reaching a new root means amending the allowlist deliberately. |
//! | [`resonance_module_does_not_reference_core_model_state`] | audit | The allowlist cannot be sidestepped by writing the path out in full: no line under the resonance modules names a prohibited root anywhere, not only in its imports, and the prohibited roots are the allowlist's complement over the crate's own module tree rather than a second list kept beside it. An import allowlist alone would police the tidy way of reaching across the boundary while leaving the untidy way open, and it is the untidy way — a fully qualified path buried mid-expression — that is likeliest to slip through review unnoticed. A hand-kept complement has the same weakness one level up: it polices the roots somebody remembered. |
//! | [`core_model_modules_do_not_import_decision_layer`] | audit | The boundary holds from the other side too: no core-model module names a decision-layer type — no channel policy, no derivation configuration, no reward parameters, no reckoning. Runtime tests can show that policy does not currently perturb a core estimate, but only the absence of the dependency makes that structurally true; a core module that could see policy at all would eventually be tempted to consult it. |
//! | [`core_and_api_paths_do_not_call_derivation`] | audit | No production path in the core, the public API, or guidance invokes the derivation function: the core produces a risk assessment and stops there. Composing an assessment with a policy is the host's decision to make explicitly, with its own channel policy and configuration in hand — a convenience call hidden inside the core would quietly reintroduce policy into the estimate every host receives, and the layering would be true only on paper. |
//! | [`steward_channel_and_test_surface_stay_sealed`] | audit | The four surfaces are the four surfaces: the steward's channel accessors and the identifier allocator are sealed at crate visibility, and every test-only export — the harness module, the numerics and signal export modules, the shared-state trait — stands behind the `test-support` feature the crate's own test builds enable and a host does not. The opacity the surface record claims is enforced structurally, so a future edit cannot quietly remove it; this scan is what makes the enforcement itself survive a merge. |
//! | [`timestamp_fields_and_default_stay_sealed`] | audit | The timestamp interface withholds the unbounded interval structurally and stays withholding it: the second and nanosecond fields and the system-time conversion are sealed at crate visibility, and the type carries no `Default`. A test cannot show a field is unreadable from outside, so the seal is checked by reading the source — the same arrangement the resonance allowlist uses for a property the visibility system enforces but a merge could quietly relax. |

//! Repository-hygiene tests: scans of the source tree.
//!
//! These tests verify properties of the codebase itself rather than
//! runtime behaviour — a cheap guardrail that flags regressions where
//! placeholder scaffolding creeps back into production source. They
//! lean on [`SourceTreeScanner`](crate::testing::SourceTreeScanner)
//! from the shared test harness so the walk / line-iteration
//! boilerplate stays in one place.
//!
//! # Cross-References
//!
//! - The algorithm records that retired the stubs these tests watch for.
//! - (´dec:derivation:allowlisted-imports´) — the derivation module is the unit the import allowlist scopes.

use crate::testing::{LineVisit, SourceTreeScanner};

/// Return true when the visited line is production source worth scanning.
fn is_scannable_source_line(visit: &LineVisit<'_>) -> bool {
    if visit.in_cfg_test {
        return false;
    }

    let trimmed = visit.line.trim_start();
    !trimmed.is_empty() && !trimmed.starts_with("//")
}

/// Return true when the visited path is under a source-tree component.
fn path_has_component(visit: &LineVisit<'_>, component: &str) -> bool {
    visit
        .path
        .components()
        .any(|path_component| path_component.as_os_str() == component)
}

/// Return true when the visited path is under any of the supplied module roots.
fn path_has_any_component(visit: &LineVisit<'_>, components: &[&str]) -> bool {
    components.iter().any(|component| path_has_component(visit, component))
}

/// Return true when the visited path is the named source file.
fn path_file_name_is(visit: &LineVisit<'_>, file_name: &str) -> bool {
    visit
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == file_name)
}

/// Strip supported `use` prefixes and return the imported `crate::<root>`.
fn crate_root_from_use_line(trimmed: &str) -> Result<Option<String>, String> {
    let rest = if let Some(rest) = trimmed.strip_prefix("use ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("pub use ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("pub(crate) use ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("pub(super) use ") {
        rest
    } else {
        return Ok(None);
    };

    let Some(after_crate) = rest.strip_prefix("crate::") else {
        return Ok(None);
    };

    let root: String = after_crate
        .chars()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect();
    if root.is_empty() {
        return Err(format!("could not parse `use crate::` target: {trimmed}"));
    }

    Ok(Some(root))
}

/// Collect forbidden token references under selected production module roots.
fn collect_token_violations(roots: &[&str], tokens: &[&str], violation_kind: &str) -> Vec<String> {
    let mut violations = Vec::new();

    SourceTreeScanner::new().skip_dir("tests").for_each_line(|visit| {
        if !is_scannable_source_line(&visit) || !path_has_any_component(&visit, roots) {
            return;
        }

        let trimmed = visit.line.trim_start();
        for token in tokens {
            if trimmed.contains(token) {
                violations.push(format!(
                    "{}:{}: prohibited {violation_kind} token `{token}` in `{trimmed}`",
                    visit.path.display(),
                    visit.line_no,
                ));
            }
        }
    });

    violations
}

/// No placeholder function survives anywhere in production source: the whole
/// source tree, minus test modules and the test subtree, contains no
/// stub-prefixed definition. Scaffolding written to make a module compile
/// before its algorithm existed is the kind of thing that quietly outlives the
/// work it was standing in for, and a returned placeholder is far harder to
/// notice than a missing function. Test code may still name stubs, because a
/// test about a stub is not a stub.
///
/// ´claim:audit:no-placeholder-function-survives-in-production-source-outside-test-modules´
/// ´test:crate:no-stub-functions-in-codebase´
#[test]
fn no_stub_functions_in_codebase() {
    let mut violations = Vec::new();

    SourceTreeScanner::new().skip_dir("tests").for_each_line(|visit| {
        if !is_scannable_source_line(&visit) {
            return;
        }
        let trimmed = visit.line.trim();
        if trimmed.contains("fn stub_") && (trimmed.starts_with("pub") || trimmed.starts_with("fn")) {
            violations.push(format!("{}:{}: {}", visit.path.display(), visit.line_no, trimmed));
        }
    });

    assert!(
        violations.is_empty(),
        "Found stub function definitions that should be removed:\n{}",
        violations.join("\n")
    );
}

/// The module that once held the crate's placeholders is absent from the
/// source tree and stays absent. A deletion is not self-enforcing: a merge, a
/// revert, or a well-meaning restoration could bring the file back, and its
/// mere existence would give new placeholder code an obvious home to be added
/// to.
///
/// ´claim:audit:the-deleted-placeholder-module-stays-deleted´
/// ´test:crate:stubs-module-deleted´
#[test]
fn stubs_module_deleted() {
    let stubs_path = SourceTreeScanner::new().root().join("stubs.rs");
    assert!(
        !stubs_path.exists(),
        "src/stubs.rs should remain deleted after Layer 4 completion"
    );
}

/// Permitted `crate::<segment>` roots for imports inside `src/resonance/`.
///
/// Derived verbatim from the allowlist the derivation module is scoped by
/// (´dec:derivation:allowlisted-imports´).
/// `resonance` is included so the derivation layer can re-use its own
/// sibling modules (`channel`, `tags`, `ambiguity`, `landscape`, `derivation`).
/// Every other sibling module is prohibited by default — adding a new
/// permitted crate root requires updating this constant, which keeps
/// the test failing closed.
const RESONANCE_ALLOWED_CRATE_ROOTS: &[&str] = &[
    "types",
    "numerics",
    "config",
    "health",
    "linalg",
    "error",
    "resonance",
    "assessment",
];

/// The derivation layer's purity is a structural property of the source tree,
/// not a convention: every crate-internal import under the resonance modules
/// names a root on a closed allowlist, and any other root fails the build.
/// Purity is what makes derivation reproducible from its arguments alone, and
/// the visibility system cannot express a rule of this shape — a module is
/// either visible to the crate or not — so the boundary is checked by reading
/// the source. The list fails closed: reaching a new root means amending the
/// allowlist deliberately.
///
/// ´claim:audit:the-derivation-layer-imports-only-from-a-closed-allowlist-of-crate-roots´
/// ´test:crate:resonance-module-does-not-import-core-state´
#[test]
fn resonance_module_does_not_import_core_state() {
    let mut violations = Vec::new();

    SourceTreeScanner::new().for_each_line(|visit| {
        if !is_scannable_source_line(&visit) || !path_has_component(&visit, "resonance") {
            return;
        }

        let trimmed = visit.line.trim_start();
        let root = crate_root_from_use_line(trimmed).unwrap_or_else(|error| {
            violations.push(format!("{}:{}: {error}", visit.path.display(), visit.line_no));
            None
        });
        let Some(root) = root else { return };

        if !RESONANCE_ALLOWED_CRATE_ROOTS.contains(&root.as_str()) {
            violations.push(format!(
                "{}:{}: disallowed import `crate::{}` — dec:derivation:allowlisted-imports allowlist: {:?}",
                visit.path.display(),
                visit.line_no,
                root,
                RESONANCE_ALLOWED_CRATE_ROOTS,
            ));
        }
    });

    assert!(
        violations.is_empty(),
        "Prohibited imports inside src/resonance/ (dec:derivation:allowlisted-imports \
         module boundary enforcement):\n{}",
        violations.join("\n")
    );
}

/// The crate's module roots, read off the source tree rather than listed: each
/// directory under `src/` and each module file beside them.
///
/// `lib` is the crate root rather than a module of it, and `tests` is the
/// crate's own test tree, which the scans skip.
fn crate_module_roots() -> Vec<String> {
    let src = SourceTreeScanner::new().root().to_path_buf();
    let mut roots: Vec<String> = std::fs::read_dir(&src)
        .expect("the crate's src/ is readable")
        .map(|entry| entry.expect("a readable entry under src/"))
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.path().is_dir() {
                Some(name)
            } else {
                name.strip_suffix(".rs").map(str::to_owned)
            }
        })
        .filter(|name| name != "lib" && name != "tests")
        .collect();
    roots.sort_unstable();
    roots.dedup();
    roots
}

/// The roots the allowlist prohibits: its complement over the crate's module
/// roots, as fully-qualified path prefixes.
///
/// Derived rather than maintained beside the allowlist. A second list kept by
/// hand is a second thing to remember, and what it forgot was six roots — the
/// public interface, the metrics, the persistence, the report, the codec and
/// the signal modules, three of which hold exactly the state the boundary
/// exists to keep out.
fn resonance_forbidden_roots() -> Vec<String> {
    crate_module_roots()
        .into_iter()
        .filter(|root| !RESONANCE_ALLOWED_CRATE_ROOTS.contains(&root.as_str()))
        .map(|root| format!("crate::{root}"))
        .collect()
}

/// The allowlist cannot be sidestepped by writing the path out in full: no
/// line under the resonance modules names a prohibited root anywhere, not only
/// in its imports, and the prohibited roots are the allowlist's complement over
/// the crate's own module tree rather than a second list kept beside it. An
/// import allowlist alone would police the tidy way of reaching across the
/// boundary while leaving the untidy way open, and it is the untidy way — a
/// fully qualified path buried mid-expression — that is likeliest to slip
/// through review unnoticed. A hand-kept complement has the same weakness one
/// level up: it polices the roots somebody remembered.
///
/// ´claim:audit:the-import-allowlist-cannot-be-sidestepped-by-a-fully-qualified-path´
/// ´test:crate:resonance-module-does-not-reference-core-model-state´
#[test]
fn resonance_module_does_not_reference_core_model_state() {
    let forbidden = resonance_forbidden_roots();
    // Fails closed: an enumeration that returned nothing would make the guard
    // vacuous, and a permitted root that has been deleted or renamed would
    // leave the allowlist naming something that no longer exists.
    let roots = crate_module_roots();
    assert!(
        forbidden.len() + RESONANCE_ALLOWED_CRATE_ROOTS.len() == roots.len(),
        "every permitted root must be a module root: allowlist {RESONANCE_ALLOWED_CRATE_ROOTS:?}, tree {roots:?}"
    );

    let forbidden_refs: Vec<&str> = forbidden.iter().map(String::as_str).collect();
    let violations = collect_token_violations(&["resonance"], &forbidden_refs, "core-model reference");

    assert!(
        violations.is_empty(),
        "Resonance decision-layer modules must not reference roots outside the dec:derivation:allowlisted-imports \
         allowlist {RESONANCE_ALLOWED_CRATE_ROOTS:?} directly:\n{}",
        violations.join("\n")
    );
}

/// Production module roots that belong to the core model / assessment side of
/// the boundary where the Core's work ends (´dec:ordering:core-boundary´).
const CORE_MODEL_ROOTS: &[&str] = &["extraction", "feature", "guidance", "identity", "ledger", "model", "risk"];

/// Decision-layer tokens that must not appear in core-model source modules.
const DECISION_LAYER_TOKENS: &[&str] = &[
    "crate::resonance",
    "ChannelPolicy",
    "DerivationConfig",
    "RewardParameters",
    "Reckoning",
    "ResonanceInput",
    "ResonanceProfile",
    "compute_derived_constants",
    "derive_reckoning",
    "derive_resonance",
];

/// The boundary holds from the other side too: no core-model module names a
/// decision-layer type — no channel policy, no derivation configuration, no
/// reward parameters, no reckoning. Runtime tests can show that policy does not
/// currently perturb a core estimate, but only the absence of the dependency
/// makes that structurally true; a core module that could see policy at all
/// would eventually be tempted to consult it.
///
/// ´claim:audit:the-layer-boundary-holds-in-both-directions-with-core-modules-naming-no-decision-layer-type´
/// ´test:crate:core-model-modules-do-not-import-decision-layer´
#[test]
fn core_model_modules_do_not_import_decision_layer() {
    let violations = collect_token_violations(CORE_MODEL_ROOTS, DECISION_LAYER_TOKENS, "decision-layer");

    assert!(
        violations.is_empty(),
        "Core-model modules must not import or reference decision-layer policy/derivation types:\n{}",
        violations.join("\n")
    );
}

/// Derivation Function tokens that must not appear in Core/API/guidance production
/// paths. Tests and comments may compose Core assessments with derivation
/// explicitly, but live Core code must stop at `RiskAssessment`.
const CORE_API_DERIVATION_TOKENS: &[&str] = &["derive_reckoning", "derive_resonance", "ResonanceInput"];

/// Return true when a path is a Core/API read-path source file covered by the
/// no-implicit-derivation criterion (´dec:operational:core-derivation-split´).
fn is_core_or_api_derivation_forbidden_path(visit: &LineVisit<'_>) -> bool {
    path_has_component(visit, "api")
        || path_has_component(visit, "assessment")
        || path_has_component(visit, "guidance")
        || path_file_name_is(visit, "assessment.rs")
}

/// No production path in the core, the public API, or guidance invokes the
/// derivation function: the core produces a risk assessment and stops there.
/// Composing an assessment with a policy is the host's decision to make
/// explicitly, with its own channel policy and configuration in hand — a
/// convenience call hidden inside the core would quietly reintroduce policy
/// into the estimate every host receives, and the layering would be true only
/// on paper.
///
/// ´claim:audit:no-core-or-api-path-invokes-the-derivation-function-so-composition-stays-the-hosts-to-make´
/// ´test:crate:core-and-api-paths-do-not-call-derivation´
#[test]
fn core_and_api_paths_do_not_call_derivation() {
    let mut violations = Vec::new();

    SourceTreeScanner::new().skip_dir("tests").for_each_line(|visit| {
        if !is_scannable_source_line(&visit) || !is_core_or_api_derivation_forbidden_path(&visit) {
            return;
        }

        let trimmed = visit.line.trim_start();
        for token in CORE_API_DERIVATION_TOKENS {
            if trimmed.contains(token) {
                violations.push(format!(
                    "{}:{}: Derivation Function token `{token}` in Core/API path: `{trimmed}`",
                    visit.path.display(),
                    visit.line_no,
                ));
            }
        }
    });

    assert!(
        violations.is_empty(),
        "Core/API production paths must not call the Derivation Function implicitly:\n{}",
        violations.join("\n")
    );
}

/// The four surfaces are the four surfaces: the steward's channel
/// accessors and the identifier allocator are sealed at crate
/// visibility, and every test-only export — the harness module, the
/// numerics and signal export modules, the shared-state trait — stands
/// behind the `test-support` feature the crate's own test builds enable
/// and a host does not. The opacity the surface record claims is
/// enforced structurally, so a future edit cannot quietly remove it;
/// this scan is what makes the enforcement itself survive a merge.
///
/// ´claim:audit:the-steward-channel-accessors-stay-sealed-and-the-test-surface-stays-behind-its-feature´
/// ´test:crate:steward-channel-and-test-surface-stay-sealed´
#[test]
fn steward_channel_and_test_surface_stay_sealed() {
    let lib_rs = SourceTreeScanner::new().root().join("lib.rs");
    let source = std::fs::read_to_string(&lib_rs).expect("src/lib.rs is readable");

    // The four accessors the audit found public are crate-internal.
    for sealed in [
        "pub(crate) const fn shared(",
        "pub(crate) const fn command_tx(",
        "pub(crate) const fn label_tx(",
        "pub(crate) fn next_assessment_id(",
    ] {
        assert!(source.contains(sealed), "accessor stays sealed at crate visibility: {sealed}");
    }

    // Each test-only export stands directly under the feature gate.
    let gate = "#[cfg(any(test, feature = \"test-support\"))]";
    for gated in [
        "pub mod testing;",
        "pub mod numerics_export {",
        "pub mod signal_export {",
        "pub use assessment::AssessmentSharedState;",
    ] {
        let position = source.find(gated).unwrap_or_else(|| panic!("export present: {gated}"));
        let preceding = &source[..position];
        let gate_position = preceding
            .rfind(gate)
            .unwrap_or_else(|| panic!("gate present before: {gated}"));
        assert!(
            position - gate_position < 400,
            "the gate stands directly over its export: {gated}"
        );
    }
}

/// The timestamp interface withholds the unbounded interval structurally
/// and stays withholding it: the second and nanosecond fields and the
/// system-time conversion are sealed at crate visibility, and the type
/// carries no `Default`. A test cannot show a field is unreadable from
/// outside, so the seal is checked by reading the source — the same
/// arrangement the resonance allowlist uses for a property the
/// visibility system enforces but a merge could quietly relax.
///
/// ´claim:audit:the-timestamps-fields-conversion-and-default-stay-sealed´
/// ´test:crate:timestamp-fields-and-default-stay-sealed´
#[test]
fn timestamp_fields_and_default_stay_sealed() {
    let types_rs = SourceTreeScanner::new().root().join("types.rs");
    let source = std::fs::read_to_string(&types_rs).expect("src/types.rs is readable");

    assert!(
        source.contains("pub(crate) seconds: i64"),
        "the seconds field stays sealed at crate visibility"
    );
    assert!(
        source.contains("pub(crate) nanos: u32"),
        "the nanos field stays sealed at crate visibility"
    );
    assert!(
        source.contains("pub(crate) fn to_system_time"),
        "the system-time conversion stays sealed at crate visibility"
    );
    assert!(
        !source.contains("impl Default for PersistentTimestamp"),
        "the timestamp carries no Default: obtaining the present requires saying so"
    );
}
