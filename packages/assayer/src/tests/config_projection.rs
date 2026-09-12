// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`configuration_tables_project_onto_the_live_declarations`] | config | Every row of the specification's sixteen configuration tables meets a live package declaration, and every parameter those declarations carry meets a row or a stated reason, with neither side written down: the rows are read from the chapter's own source and the parameters from the `Default` bodies the package declares, corroborated by serde's own view of the top-level configuration. Each bound row is then checked twice — its tabulated value against the shipped one and the shipped one against the tabulated domain — so a default that moved and a default that left its constraint are separate findings rather than one. A hand count of this agreement is what the chapter says it will not keep (´chap:spec:configuration´), and a test holding its own copy of either side would keep passing through exactly the edit it exists to catch. |
//! | [`configuration_projection_refuses_value_domain_and_exposure_drift`] | config | The projection refuses each of the three ways configuration can drift, one fabricated fixture at a time and every bound row load-bearing: a shipped default moved off its tabulated value is refused and named, a shipped default pushed outside its tabulated domain is refused as a domain finding rather than as a value one, and exposure drift is refused in all three directions it takes — a package parameter no row and no stated reason accounts for, a declared surface the package no longer carries, and a table row no binding accounts for. A guardrail that only checked the live pair would pass on the day its own oracle stopped working. |
//! | [`configuration_projection_accounts_for_a_declared_departure`] | config | A departure from a tabulated value is declared rather than exempted: the projection accepts a row whose package declaration deliberately differs from the specification only while that difference is actually there, and refuses the declaration the moment the package agrees with the table again. The chain-length normaliser is the row the chapter documents a departure for (´tab:config:extraction´), and against the live package the departure is absent — the shipped divisor is the tabulated one at both construction sites — so the row binds as an ordinary agreement and the mechanism is exercised against a fabricated package instead. An exemption comment would have gone on excusing a difference that had already been repaired. |

#![allow(clippy::float_cmp)]

//! Crate-level tests projecting the specification's configuration chapter onto
//! the package declarations it governs (´chap:spec:configuration´).
//!
//! The chapter is sixteen tables of parameters, and each row states a default
//! and a constraint that the package is supposed to ship. Nothing in the
//! package reads those tables, so agreement between the two has no mechanism
//! behind it — which is why the chapter declines to keep a hand count of it and
//! asks for a projection instead. This module is that projection: it discovers
//! the rows from the chapter's own source, discovers the parameters from the
//! `Default` bodies and named constants the package declares, and requires the
//! two to correspond row by row, with every unbound row and every unclaimed
//! parameter a refusal rather than a silence.

use std::collections::{BTreeMap, BTreeSet};

// ═══════════════════════════════════════════════════════════════════════════════
// The two sources, pulled in as build inputs
// ═══════════════════════════════════════════════════════════════════════════════

/// The configuration chapter's own source. Pulled in with `include_str!`, so
/// the specification part is a build input of this test: editing a table
/// rebuilds and re-discovers.
const SPEC_CONFIGURATION: &str = include_str!("../../docs/spec/reference-configuration.md");

/// The package sources that declare configuration. Each is a build input for
/// the same reason: a field added to or removed from a `Default` body changes
/// the discovered set on the next build with nothing here to update.
const SOURCES: &[(&str, &str)] = &[
    ("config/types.rs", include_str!("../config/types.rs")),
    ("feature/standardisation.rs", include_str!("../feature/standardisation.rs")),
    ("risk/calibration.rs", include_str!("../risk/calibration.rs")),
    ("risk/challenge.rs", include_str!("../risk/challenge.rs")),
    ("model/marginalise.rs", include_str!("../model/marginalise.rs")),
    ("linalg/bridge.rs", include_str!("../linalg/bridge.rs")),
    ("health/drift.rs", include_str!("../health/drift.rs")),
    ("health/published.rs", include_str!("../health/published.rs")),
    ("resonance/derivation.rs", include_str!("../resonance/derivation.rs")),
    ("resonance/channel.rs", include_str!("../resonance/channel.rs")),
    ("guidance/mod.rs", include_str!("../guidance/mod.rs")),
    ("extraction/mod.rs", include_str!("../extraction/mod.rs")),
    ("numerics.rs", include_str!("../numerics.rs")),
    ("owner/label_path.rs", include_str!("../owner/label_path.rs")),
];

/// The host-facing configuration declarations, each named with the source that
/// declares it. These are the types a host constructs and hands to the package:
/// the Core configuration and its sub-configurations, the per-call derivation
/// policy, the rendering call's display record, the guidance call's parameters
/// and the Sentinel extraction's configuration. Types that merely restate them
/// downstream — the label pipeline's derived configuration, for one — are not
/// declaration surfaces and are entered only where a table row names one of
/// their fields.
const ROOTS: &[(&str, &str)] = &[
    ("AssayerConfig", "config/types.rs"),
    ("ModelConfig", "config/types.rs"),
    ("EligibilityPolicy", "config/types.rs"),
    ("TemporalConfig", "config/types.rs"),
    ("HibernationConfig", "config/types.rs"),
    ("LedgerConfig", "config/types.rs"),
    ("CholeskyConfig", "config/types.rs"),
    ("ConcordanceConfig", "config/types.rs"),
    ("MonitoringConfig", "config/types.rs"),
    ("ConvergenceThresholdConfig", "config/types.rs"),
    ("InfrastructureConfig", "config/types.rs"),
    ("BlendStatisticsConfig", "config/types.rs"),
    ("StandardisationConfig", "feature/standardisation.rs"),
    ("PlattConfig", "risk/calibration.rs"),
    ("SchurConfig", "model/marginalise.rs"),
    ("ResonanceConfig", "resonance/derivation.rs"),
    ("LabelGuidanceParams", "guidance/mod.rs"),
    ("ExtractionConfig", "extraction/mod.rs"),
    ("RewardParameters", "resonance/channel.rs"),
    ("ChannelPolicy", "resonance/channel.rs"),
];

/// Returns the text of a source named in [`SOURCES`].
fn source(name: &str) -> &'static str {
    for (key, text) in SOURCES {
        if *key == name {
            return text;
        }
    }
    panic!("the projection names a source it pulled in: {name}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Values
// ═══════════════════════════════════════════════════════════════════════════════

/// A value as either side declares it.
#[derive(Clone, Debug, PartialEq)]
enum Value {
    /// A number, in whatever units its own side states.
    Number(f64),
    /// A boolean.
    Boolean(bool),
    /// A word: an enumerated choice, a structural answer, or a shape.
    Text(String),
    /// A sub-configuration reached through its own `Default`.
    Delegated(String),
    /// A symbol standing for another row's value.
    Symbol(String),
}

impl Value {
    /// The comparable form of a word: lower case, and the last path segment
    /// where the package names an enumerated variant by its full path.
    fn word(text: &str) -> String {
        let tail = text.rsplit("::").next().unwrap_or(text);
        tail.trim().to_lowercase()
    }
}

/// Reads a number written the way Rust and the tables both write them: digit
/// separators in either dialect, and an exponent where a source uses one.
fn read_number(text: &str) -> Option<f64> {
    let stripped: String = text.chars().filter(|c| *c != '_' && *c != ',').collect();
    if stripped.is_empty() {
        return None;
    }
    stripped.parse::<f64>().ok()
}

/// Reads the specification's power-of-ten form, which is how the tables write
/// every value a plain decimal would render unreadably.
fn read_power_of_ten(text: &str) -> Option<f64> {
    let rest = text.strip_prefix("10^")?;
    let exponent = rest.strip_prefix('{').and_then(|r| r.strip_suffix('}')).unwrap_or(rest);
    let exponent: f64 = exponent.parse().ok()?;
    Some(10.0_f64.powf(exponent))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Discovery: the specification side
// ═══════════════════════════════════════════════════════════════════════════════

/// One row of one configuration table, as the chapter's source writes it.
#[derive(Clone, Debug)]
struct SpecRow {
    table: String,
    parameter: String,
    default_cell: String,
    constraint_cell: String,
}

impl SpecRow {
    /// The row's identity: the pair of its table and its parameter, which is
    /// what the chapter's own citation convention says a parameter is
    /// (´conv:config:parameter-citation´).
    fn key(&self) -> String {
        format!("{}|{}", self.table, self.parameter)
    }
}

/// The rows of every configuration table, read off the chapter's source rather
/// than off a transcription of it. A row added to or removed from any table
/// changes this set on the next build.
fn spec_rows() -> Vec<SpecRow> {
    let mut rows = Vec::new();
    let mut table: Option<String> = None;
    let mut in_table = false;

    for line in SPEC_CONFIGURATION.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("**Table (") {
            table = label_in_backticks(trimmed).filter(|label| label.starts_with("tab:config:"));
            in_table = false;
            continue;
        }
        if !trimmed.starts_with('|') {
            if !trimmed.is_empty() {
                in_table = false;
            }
            continue;
        }
        let cells: Vec<_> = trimmed.trim_matches('|').split('|').map(str::trim).collect();
        if cells.first().copied() == Some("Parameter") {
            in_table = table.is_some();
            continue;
        }
        if !in_table || cells.first().is_some_and(|first| first.starts_with("---")) {
            continue;
        }
        let Some(name) = table.clone() else { continue };
        assert!(
            cells.len() >= 3,
            "a configuration row carries parameter, default and constraint"
        );
        rows.push(SpecRow {
            table: name,
            parameter: cells[0].to_owned(),
            default_cell: cells[1].to_owned(),
            constraint_cell: cells[2].to_owned(),
        });
    }
    rows
}

/// The label a table's heading carries, taken from between its backticks.
fn label_in_backticks(line: &str) -> Option<String> {
    let start = line.find('`')? + 1;
    let rest = &line[start..];
    let end = rest.find('`')?;
    Some(rest[..end].to_owned())
}

/// The symbols the tables define, each mapped to the value of the row that
/// carries it. A default or a bound written as another parameter's symbol is
/// resolved through this map rather than through a second copy of the number.
fn symbol_values(rows: &[SpecRow]) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    for row in rows {
        let Some(symbol) = leading_symbol(&row.parameter) else {
            continue;
        };
        if let Value::Number(value) = read_spec_value(&row.default_cell) {
            map.entry(symbol).or_insert(value);
        }
    }
    map
}

/// The mathematical symbol a parameter cell opens with, where it opens with one.
fn leading_symbol(parameter: &str) -> Option<String> {
    let rest = parameter.strip_prefix('$')?;
    let end = rest.find('$')?;
    Some(rest[..end].to_owned())
}

/// Reads a tabulated default: a power of ten, a plain or separated number
/// possibly followed by its units, a boolean, another row's symbol, or a word.
fn read_spec_value(cell: &str) -> Value {
    let bare = cell.trim().trim_matches('$').trim();
    if let Some((coefficient, _)) = bare.split_once(r"\cdot")
        && let Some(value) = read_power_of_ten(coefficient.trim())
    {
        return Value::Number(value);
    }
    if let Some(value) = read_power_of_ten(bare) {
        return Value::Number(value);
    }
    if bare == "true" || bare == "false" {
        return Value::Boolean(bare == "true");
    }
    if bare.starts_with('\\') && !bare.contains(' ') {
        return Value::Symbol(bare.to_owned());
    }
    let leading: String = bare
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();
    if let Some(value) = read_number(&leading) {
        return Value::Number(value);
    }
    Value::Text(cell.trim().to_owned())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Discovery: the package side
// ═══════════════════════════════════════════════════════════════════════════════

/// The body of the item whose header opens `text`, delimited by the braces that
/// open and close it.
///
/// Bounding the window at the closing brace is what keeps a discovery about one
/// `Default` body from taking in whatever body follows it in the same file.
fn brace_delimited_body(text: &str) -> &str {
    let open = text.find('{').expect("the scraped item has a body");
    let mut depth = 0_usize;
    for (offset, byte) in text.bytes().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &text[open + 1..offset];
                }
            }
            _ => {}
        }
    }
    panic!("the scraped item's body is closed");
}

/// The one occurrence of `needle` in `text`, refusing a needle that is not
/// unique so that a discovery can never silently take the first of several.
fn unique_occurrence<'a>(text: &'a str, needle: &str, what: &str) -> &'a str {
    let mut found = text.match_indices(needle);
    let (start, _) = found.next().unwrap_or_else(|| panic!("{what} exists"));
    assert!(found.next().is_none(), "{what} is unique");
    &text[start..]
}

/// Splits a struct-literal body at the commas that separate its fields, which
/// are the commas outside every bracket and every string.
fn split_fields(body: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut depth = 0_i32;
    let mut in_string = false;
    for line in body.lines() {
        let code = line.split_once("//").map_or(line, |(before, _)| before);
        for character in code.chars() {
            if in_string {
                current.push(character);
                if character == '"' {
                    in_string = false;
                }
                continue;
            }
            match character {
                '"' => {
                    in_string = true;
                    current.push(character);
                }
                '(' | '[' | '{' => {
                    depth += 1;
                    current.push(character);
                }
                ')' | ']' | '}' => {
                    depth -= 1;
                    current.push(character);
                }
                ',' if depth == 0 => {
                    fields.push(std::mem::take(&mut current));
                }
                _ => current.push(character),
            }
        }
        current.push(' ');
    }
    fields.push(current);
    fields
        .into_iter()
        .map(|field| field.trim().to_owned())
        .filter(|field| !field.is_empty())
        .collect()
}

/// The `field: value` pairs of the `Default` body a type declares, read off
/// that body rather than off a list kept beside it.
fn default_body_fields(source_name: &str, ty: &str) -> BTreeMap<String, String> {
    let text = source(source_name);
    let header = format!("impl Default for {ty} {{");
    let item = unique_occurrence(text, &header, &format!("the `Default` body of `{ty}`"));
    let body = brace_delimited_body(item);
    let function = unique_occurrence(body, "fn default() -> Self {", &format!("the `default` of `{ty}`"));
    let function_body = brace_delimited_body(function);
    let literal = unique_occurrence(function_body, "Self {", &format!("the struct literal of `{ty}`'s default"));
    let literal = brace_delimited_body(literal);

    let mut fields = BTreeMap::new();
    for field in split_fields(literal) {
        let Some(colon) = field.find(':') else {
            panic!("a default field names itself: {field}");
        };
        let name = field[..colon].trim().to_owned();
        let expression = field[colon + 1..].trim().to_owned();
        assert!(
            !name.is_empty() && !name.contains(' '),
            "a default field's name is one identifier: {field}"
        );
        fields.insert(name, expression);
    }
    fields
}

/// The value a named constant is declared with, read off its own declaration.
fn constant_expression(source_name: &str, name: &str) -> String {
    let text = source(source_name);
    let needle = format!("const {name}:");
    let item = unique_occurrence(text, &needle, &format!("the constant `{name}` in {source_name}"));
    let equals = item.find('=').expect("a constant is initialised");
    let end = item[equals..].find(';').expect("a constant declaration is terminated");
    item[equals + 1..equals + end].trim().to_owned()
}

/// Classifies a default expression as the package writes it: a delegation to a
/// sub-configuration's own default, a number, a boolean, or a word. An
/// expression that is a named constant is resolved through that constant's own
/// declaration in the same source, so a constant moved is a value moved.
fn read_package_value(source_name: &str, expression: &str) -> Value {
    let expression = expression.trim();
    if let Some(head) = expression.strip_suffix("::default()") {
        let ty = head.rsplit("::").next().unwrap_or(head);
        return Value::Delegated(ty.trim().to_owned());
    }
    if expression == "true" || expression == "false" {
        return Value::Boolean(expression == "true");
    }
    if let Some(value) = read_number(expression) {
        return Value::Number(value);
    }
    let identifier = expression.strip_prefix("Self::").unwrap_or(expression);
    let is_constant = !identifier.is_empty()
        && identifier
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if is_constant {
        let resolved = constant_expression(source_name, identifier);
        if let Some(value) = read_number(&resolved) {
            return Value::Number(value);
        }
        if resolved == "true" || resolved == "false" {
            return Value::Boolean(resolved == "true");
        }
        return Value::Text(resolved);
    }
    Value::Text(expression.to_owned())
}

/// Everything the package declares: the parameters of the declaration
/// surfaces, keyed by surface and field, and the delegations between them.
fn package_declarations() -> (BTreeMap<String, Value>, BTreeMap<String, String>) {
    let mut parameters = BTreeMap::new();
    let mut delegations = BTreeMap::new();
    for (ty, source_name) in ROOTS {
        for (field, expression) in default_body_fields(source_name, ty) {
            let key = format!("{ty}.{field}");
            match read_package_value(source_name, &expression) {
                Value::Delegated(target) => {
                    delegations.insert(key, target);
                }
                value => {
                    parameters.insert(key, value);
                }
            }
        }
    }
    (parameters, delegations)
}

// ═══════════════════════════════════════════════════════════════════════════════
// The binding: which declaration each row projects onto
// ═══════════════════════════════════════════════════════════════════════════════

/// How a tabulated row reaches the package.
///
/// The binding is the correspondence and nothing else: it holds no value and no
/// domain, both of which are discovered from the two sides. A binding that
/// names a row no table carries, or a declaration the package no longer
/// carries, is refused rather than ignored.
#[derive(Clone, Copy, Debug)]
enum Binding {
    /// A field of a declaration surface. `scale` converts the tabulated units
    /// into the field's own, and is one wherever the two agree.
    Field {
        root: &'static str,
        field: &'static str,
        scale: f64,
    },
    /// A named constant: shipped at the tabulated value, but not settable.
    Constant { source: &'static str, name: &'static str },
    /// A field of a derived internal declaration: shipped, with no host surface.
    Internal {
        source: &'static str,
        ty: &'static str,
        field: &'static str,
        note: &'static str,
    },
    /// Supplied per registration, so there is no package-side default to project.
    Registration { locus: &'static str },
    /// A property of the design rather than a value a host sets.
    Structural { note: &'static str },
    /// No configuration surface ships for this row.
    Absent { note: &'static str },
    /// A declared departure: the package deliberately ships a different value
    /// from the tabulated one, and the projection requires that difference to
    /// actually be there.
    Departure {
        root: &'static str,
        field: &'static str,
        note: &'static str,
    },
}

impl Binding {
    /// A field of a declaration surface, in the table's own units.
    const fn field(root: &'static str, field: &'static str) -> Self {
        Self::Field { root, field, scale: 1.0 }
    }

    /// The key under which the shipped value of this binding is looked up.
    fn shipped_key(self) -> Option<String> {
        match self {
            Self::Field { root, field, .. } | Self::Departure { root, field, .. } => Some(format!("{root}.{field}")),
            Self::Constant { source, name } => Some(format!("{source}::{name}")),
            Self::Internal { ty, field, .. } => Some(format!("{ty}.{field}")),
            Self::Registration { .. } | Self::Structural { .. } | Self::Absent { .. } => None,
        }
    }

    /// The account a binding gives of a row that reaches no field of a
    /// declaration surface: the constant that carries it, the registration that
    /// supplies it, the design property it states, or the reason nothing ships.
    /// A row projecting onto no field is accounted for by this sentence or it is
    /// not accounted for at all.
    const fn account(self) -> Option<&'static str> {
        match self {
            Self::Field { .. } => None,
            Self::Constant { name, .. } => Some(name),
            Self::Registration { locus } => Some(locus),
            Self::Internal { note, .. } | Self::Structural { note } | Self::Absent { note } | Self::Departure { note, .. } => {
                Some(note)
            }
        }
    }

    /// The declaration-surface parameter this binding claims, where it claims one.
    fn claimed_parameter(self) -> Option<String> {
        match self {
            Self::Field { root, field, .. } | Self::Departure { root, field, .. } => Some(format!("{root}.{field}")),
            _ => None,
        }
    }
}

/// The correspondence, row by row. Every entry names a row of the chapter and
/// the declaration it projects onto; the projection requires the two sets to
/// cover each other exactly, so an added table row and a stale entry are both
/// refusals.
fn bindings() -> BTreeMap<String, Binding> {
    const ROWS: &[(&str, &str, Binding)] = &[
        // ── Core risk model parameters ──────────────────────────────────────
        (
            "tab:config:risk-model",
            r"$\gamma_\text{opr}$ (operational forgetting)",
            Binding::field("ModelConfig", "gamma_opr"),
        ),
        (
            "tab:config:risk-model",
            r"$\gamma_\text{inh}$ (sister and anchor forgetting)",
            Binding::field("ModelConfig", "gamma_inh"),
        ),
        (
            "tab:config:risk-model",
            r"$\lambda_\text{prior}$ (prior precision)",
            Binding::field("ModelConfig", "lambda_prior"),
        ),
        (
            "tab:config:risk-model",
            r"$\lambda_\text{floor}$ (prior replenishment)",
            Binding::field("ModelConfig", "lambda_floor"),
        ),
        (
            "tab:config:risk-model",
            r"$c$ (leverage safety factor)",
            Binding::field("ModelConfig", "c_leverage"),
        ),
        (
            "tab:config:risk-model",
            r"$N_\text{recompute}$ (Cholesky recomputation interval)",
            Binding::field("CholeskyConfig", "n_recompute"),
        ),
        (
            "tab:config:risk-model",
            r"$\kappa_\text{growth}$ (conditioning-growth trigger)",
            Binding::field("CholeskyConfig", "kappa_growth_factor"),
        ),
        (
            "tab:config:risk-model",
            r"$\varepsilon_\text{Schur}$ (Schur regularisation)",
            Binding::field("SchurConfig", "epsilon_schur"),
        ),
        (
            "tab:config:risk-model",
            r"$\kappa_\text{posture}$ (host posture ceiling on a removed block)",
            Binding::field("SchurConfig", "posture_condition_ceiling"),
        ),
        (
            "tab:config:risk-model",
            r"$\gamma_\kappa$ (feature compression rate)",
            Binding::field("ModelConfig", "gamma_kappa"),
        ),
        (
            "tab:config:risk-model",
            r"$w_\text{ceiling}$ (importance weight ceiling)",
            Binding::field("ModelConfig", "w_ceiling"),
        ),
        (
            "tab:config:risk-model",
            r"$P_{+,0}$ (initial positive-valence rate)",
            Binding::field("ModelConfig", "p_plus_init"),
        ),
        // ── The eligibility policy ──────────────────────────────────────────
        (
            "tab:config:eligibility",
            r"Challenge failure counts as unconfounded",
            Binding::field("EligibilityPolicy", "challenge_fail_is_unconfounded"),
        ),
        // ── Outcome axis per-axis defaults ──────────────────────────────────
        (
            "tab:config:axis",
            r"$\gamma_a$ (axis forgetting)",
            Binding::Registration {
                locus: "OutcomeAxisRegistration::gamma (src/owner/commands.rs)",
            },
        ),
        (
            "tab:config:axis",
            r"$\kappa_{a,0}$ (initial compression scale)",
            Binding::Registration {
                locus: "OutcomeAxisRegistration::initial_kappa (src/owner/commands.rs)",
            },
        ),
        (
            "tab:config:axis",
            r"Training eligibility mode",
            Binding::Registration {
                locus: "OutcomeAxisRegistration::eligibility (src/owner/commands.rs)",
            },
        ),
        (
            "tab:config:axis",
            r"Spatial features",
            Binding::Registration {
                locus: "OutcomeAxisRegistration::spatial_features (src/owner/commands.rs)",
            },
        ),
        // ── Temporal parameters ─────────────────────────────────────────────
        (
            "tab:config:temporal",
            r"$\gamma_{t,\text{core}}$ (core model time decay)",
            Binding::field("TemporalConfig", "gamma_t_core"),
        ),
        (
            "tab:config:temporal",
            r"$\gamma_{t,L}$ (Ledger time decay)",
            Binding::field("TemporalConfig", "gamma_t_ledger"),
        ),
        (
            "tab:config:temporal",
            r"$\gamma_{t,\text{id}}$ (identity cell time decay)",
            Binding::field("TemporalConfig", "gamma_t_identity"),
        ),
        // ── Identity layer parameters ───────────────────────────────────────
        (
            "tab:config:identity",
            r"$L_d$ (competitive depth cutoff)",
            Binding::Registration {
                locus: "IdentityDimensionRegistration::depth_cutoff (src/owner/commands.rs)",
            },
        ),
        (
            "tab:config:identity",
            r"$\lambda_\text{id}$ (outcome EWMA rate)",
            Binding::Internal {
                source: "owner/label_path.rs",
                ty: "LabelPipelineConfig",
                field: "lambda_identity",
                note: "shipped by the label pipeline's own declaration; no host surface carries it",
            },
        ),
        (
            "tab:config:identity",
            r"$\lambda_m$ (measurement EWMA rate)",
            Binding::Absent {
                note: "no declaration and no production update site carries the measurement smoothing factor",
            },
        ),
        (
            "tab:config:identity",
            r"Signal cache capacity",
            Binding::field("InfrastructureConfig", "signal_cache_capacity"),
        ),
        (
            "tab:config:identity",
            r"Graph budget per dimension",
            Binding::Registration {
                locus: "IdentityDimensionRegistration::budget (src/owner/commands.rs)",
            },
        ),
        (
            "tab:config:identity",
            r"Split threshold per dimension",
            Binding::Registration {
                locus: "IdentityDimensionRegistration::budget (src/owner/commands.rs)",
            },
        ),
        // ── Ledger parameters ───────────────────────────────────────────────
        (
            "tab:config:ledger",
            r"$\lambda_L$ (Ledger EWMA rate)",
            Binding::field("LedgerConfig", "lambda_l"),
        ),
        (
            "tab:config:ledger",
            r"$\gamma_{t,L}$ (time-indexed decay)",
            Binding::field("TemporalConfig", "gamma_t_ledger"),
        ),
        (
            "tab:config:ledger",
            r"$N_\text{absent}$ (consecutive-absence deletion threshold)",
            Binding::field("LedgerConfig", "n_absent"),
        ),
        (
            "tab:config:ledger",
            r"$N_\text{ledger}$ (recent eligible label window)",
            Binding::Constant {
                source: "guidance/mod.rs",
                name: "RECENT_ELIGIBLE_LABEL_WINDOW",
            },
        ),
        (
            "tab:config:ledger",
            r"Collection floor on the entry average",
            Binding::field("LedgerConfig", "gc_floor"),
        ),
        (
            "tab:config:ledger",
            r"Collection horizon",
            Binding::field("LedgerConfig", "gc_horizon_days"),
        ),
        // ── Calibration parameters ──────────────────────────────────────────
        (
            "tab:config:calibration",
            r"$N_\text{cal,buf}$ (buffer capacity)",
            Binding::field("PlattConfig", "n_cal_buf"),
        ),
        (
            "tab:config:calibration",
            r"$N_\text{refit}$ (periodic refit cadence)",
            Binding::field("PlattConfig", "n_refit"),
        ),
        (
            "tab:config:calibration",
            r"$N_\text{cal,min}$ (minimum records per regime)",
            Binding::field("PlattConfig", "n_cal_min"),
        ),
        (
            "tab:config:calibration",
            r"$n_\text{cal,pos}$ (minimum positive outcomes)",
            Binding::field("PlattConfig", "n_cal_pos"),
        ),
        (
            "tab:config:calibration",
            r"$n_\text{cal,neg}$ (minimum negative outcomes)",
            Binding::field("PlattConfig", "n_cal_neg"),
        ),
        (
            "tab:config:calibration",
            r"$\kappa_0$ (initial calibration parameter)",
            Binding::field("PlattConfig", "kappa_initial"),
        ),
        (
            "tab:config:calibration",
            r"$\kappa_\text{min}$ (search lower bound)",
            Binding::field("PlattConfig", "kappa_min"),
        ),
        (
            "tab:config:calibration",
            r"$\kappa_\text{max}$ (search upper bound)",
            Binding::field("PlattConfig", "kappa_max"),
        ),
        (
            "tab:config:calibration",
            r"$\gamma_\text{cal}$ (recency weighting)",
            Binding::field("PlattConfig", "gamma_cal"),
        ),
        (
            "tab:config:calibration",
            r"$s_\kappa$ (soft transition steepness)",
            Binding::Constant {
                source: "numerics.rs",
                name: "S_KAPPA",
            },
        ),
        (
            "tab:config:calibration",
            r"$\delta_\text{cal}$ (drift-reset significance)",
            Binding::field("PlattConfig", "delta_cal_threshold"),
        ),
        // ── Standardisation parameters ──────────────────────────────────────
        (
            "tab:config:standardisation",
            r"$\gamma_\text{std}$ (standardisation EWMA rate)",
            Binding::field("StandardisationConfig", "gamma_std"),
        ),
        (
            "tab:config:standardisation",
            r"$v_\text{floor}$ (variance floor)",
            Binding::field("StandardisationConfig", "v_floor"),
        ),
        (
            "tab:config:standardisation",
            r"$n_\text{std}$ (feature clip width)",
            Binding::field("StandardisationConfig", "n_clip"),
        ),
        (
            "tab:config:standardisation",
            r"$N_\text{init}$ (cold prior-mass ramp horizon)",
            Binding::field("StandardisationConfig", "n_init"),
        ),
        (
            "tab:config:standardisation",
            r"$N_\text{boot}$ (per-Sentinel bootstrap sample count)",
            Binding::field("StandardisationConfig", "n_boot"),
        ),
        (
            "tab:config:standardisation",
            r"$\alpha_\text{boot}$ (bootstrap blend factor)",
            Binding::field("StandardisationConfig", "alpha_boot"),
        ),
        // ── Concurrency parameters ──────────────────────────────────────────
        (
            "tab:config:concurrency",
            r"Label queue capacity",
            Binding::field("InfrastructureConfig", "label_channel_capacity"),
        ),
        (
            "tab:config:concurrency",
            r"Label queue overflow policy",
            Binding::Structural {
                note: "the non-blocking discipline's consequence, not a value a host sets",
            },
        ),
        (
            "tab:config:concurrency",
            r"Publication interval, in labels per publish",
            Binding::Absent {
                note: "no field of that name exists on any configuration surface",
            },
        ),
        (
            "tab:config:concurrency",
            r"Identity dimension lock granularity",
            Binding::Structural {
                note: "a property of the design's locking, not a value a host sets",
            },
        ),
        (
            "tab:config:concurrency",
            r"Deferred identity queue capacity",
            Binding::Structural {
                note: "tabulated as a shape — bounded, drop oldest — rather than as a number",
            },
        ),
        // ── Pending buffer parameters ───────────────────────────────────────
        (
            "tab:config:pending-buffer",
            r"$R$ (expected peak request rate)",
            Binding::field("InfrastructureConfig", "expected_peak_request_rate"),
        ),
        (
            "tab:config:pending-buffer",
            r"$L$ (expected median label latency)",
            Binding::field("InfrastructureConfig", "expected_label_latency_secs"),
        ),
        (
            "tab:config:pending-buffer",
            r"Buffer capacity",
            Binding::Structural {
                note: "tabulated as the computation over R and L rather than as a stored default",
            },
        ),
        (
            "tab:config:pending-buffer",
            r"Expiry horizon",
            Binding::Field {
                root: "InfrastructureConfig",
                field: "expiry_horizon_secs",
                scale: 3600.0,
            },
        ),
        (
            "tab:config:pending-buffer",
            r"Feature storage precision",
            Binding::field("InfrastructureConfig", "feature_storage_precision"),
        ),
        // ── Extraction parameters ───────────────────────────────────────────
        (
            "tab:config:extraction",
            r"$D_\text{chain}$ (chain length normalisation)",
            Binding::field("ExtractionConfig", "d_chain_norm"),
        ),
        // ── Label guidance parameters ───────────────────────────────────────
        (
            "tab:config:guidance",
            r"Default scan limit",
            Binding::field("LabelGuidanceParams", "scan_limit"),
        ),
        (
            "tab:config:guidance",
            r"Starvation score threshold",
            Binding::field("LabelGuidanceParams", "starvation_threshold"),
        ),
        // ── Health monitoring parameters ────────────────────────────────────
        (
            "tab:config:monitoring",
            r"$\kappa_\text{drift}$ (drift noise allowance)",
            Binding::field("MonitoringConfig", "kappa_drift"),
        ),
        (
            "tab:config:monitoring",
            r"$h$ (accumulator threshold)",
            Binding::field("MonitoringConfig", "h_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$N_\text{auc,min}$ (minimum class count)",
            Binding::field("MonitoringConfig", "min_auc_class_count"),
        ),
        (
            "tab:config:monitoring",
            r"$N_\text{recent}$ (recent discrimination window)",
            Binding::field("MonitoringConfig", "recent_discrimination_window"),
        ),
        (
            "tab:config:monitoring",
            r"$\epsilon_\text{sync,thresh}$ (synchronisation threshold)",
            Binding::field("MonitoringConfig", "sync_error_threshold_per_dimension"),
        ),
        (
            "tab:config:monitoring",
            r"$N_\text{conc,min}$ (minimum labels for concordance)",
            Binding::field("MonitoringConfig", "min_concordance_labels"),
        ),
        (
            "tab:config:monitoring",
            r"$\delta_\text{conc}$ (concordance deficit threshold)",
            Binding::field("MonitoringConfig", "concordance_deficit_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$\theta_\text{alarm}$ (strong alarm threshold)",
            Binding::field("MonitoringConfig", "strong_alarm_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$\theta_\text{quiet}$ (no-alarm threshold)",
            Binding::field("MonitoringConfig", "quiet_alarm_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$\kappa_\text{lab}$ (alarm-outcome noise allowance)",
            Binding::field("MonitoringConfig", "alarm_outcome_noise_allowance"),
        ),
        (
            "tab:config:monitoring",
            r"$\theta_\text{stability}$ (feature stability threshold)",
            Binding::field("MonitoringConfig", "feature_stability_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$\eta_\text{mature}$ (maturity threshold)",
            Binding::field("MonitoringConfig", "maturity_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$N_\text{adequate}$ (resolution adequacy threshold)",
            Binding::field("MonitoringConfig", "resolution_adequacy_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$\gamma_\text{latency}$ (feedback latency EWMA rate)",
            Binding::field("MonitoringConfig", "feedback_latency_ewma_rate"),
        ),
        (
            "tab:config:monitoring",
            r"$N_\text{material}$ (Ledger materiality threshold)",
            Binding::field("MonitoringConfig", "ledger_materiality_threshold"),
        ),
        (
            "tab:config:monitoring",
            r"$A_\text{min}$ (attenuation materiality floor)",
            Binding::field("MonitoringConfig", "attenuation_materiality_floor"),
        ),
        // ── Derivation function configuration ───────────────────────────────
        (
            "tab:config:derivation",
            r"$R_\text{pass}$ (opportunity cost of denial)",
            Binding::field("RewardParameters", "pass"),
        ),
        (
            "tab:config:derivation",
            r"$R_\text{friction}$ (cost of challenging a good source)",
            Binding::field("RewardParameters", "friction"),
        ),
        (
            "tab:config:derivation",
            r"$R_\text{missed}$ (cost of missing an adverse outcome)",
            Binding::field("RewardParameters", "missed"),
        ),
        (
            "tab:config:derivation",
            r"$R_\text{caught}$ (reward for identifying an adverse source)",
            Binding::field("RewardParameters", "caught"),
        ),
        (
            "tab:config:derivation",
            r"$R_\text{blocked}$ (cost of blocking a good source)",
            Binding::field("RewardParameters", "blocked"),
        ),
        (
            "tab:config:derivation",
            r"$\alpha_s$ (throttle severity fraction)",
            Binding::field("RewardParameters", "slow_severity"),
        ),
        (
            "tab:config:derivation",
            r"$\beta_c$ (block catches fraction)",
            Binding::field("RewardParameters", "block_catches"),
        ),
        (
            "tab:config:derivation",
            r"$\beta_b$ (adverse-class sensitivity)",
            Binding::field("RewardParameters", "beta_bad"),
        ),
        (
            "tab:config:derivation",
            r"$\beta_g$ (benign-class sensitivity)",
            Binding::field("RewardParameters", "beta_good"),
        ),
        (
            "tab:config:derivation",
            r"Neutral zone half-width",
            Binding::field("ChannelPolicy", "neutral_zone"),
        ),
        // ── Rendering configuration ─────────────────────────────────────────
        (
            "tab:config:rendering",
            r"$c_Q$ (bandwidth scale factor)",
            Binding::field("ResonanceConfig", "c_q"),
        ),
        (
            "tab:config:rendering",
            r"$c_{\text{susp},Q}$ (Suspicious bandwidth)",
            Binding::field("ResonanceConfig", "c_susp_q"),
        ),
        (
            "tab:config:rendering",
            r"$Q_\text{floor}$ (display bandwidth floor)",
            Binding::field("ResonanceConfig", "q_floor"),
        ),
        (
            "tab:config:rendering",
            r"$c_A$ (uncertainty attenuation coefficient)",
            Binding::field("ResonanceConfig", "c_a"),
        ),
        (
            "tab:config:rendering",
            r"$c_\ell$ (classification location compression)",
            Binding::field("ResonanceConfig", "c_ell"),
        ),
        (
            "tab:config:rendering",
            r"$c_\text{susp}$ (Suspicious magnitude coefficient)",
            Binding::field("ResonanceConfig", "c_susp"),
        ),
        (
            "tab:config:rendering",
            r"$\varepsilon_\text{mono}$ (dominated tag magnitude)",
            Binding::field("ResonanceConfig", "epsilon_mono"),
        ),
        (
            "tab:config:rendering",
            r"$Q_\text{min}$ (dominated tag bandwidth)",
            Binding::field("ResonanceConfig", "q_min"),
        ),
        // ── Companion tracker configuration ─────────────────────────────────
        (
            "tab:config:companion",
            r"$\alpha_0$ (prior failure pseudo-count)",
            Binding::Constant {
                source: "risk/challenge.rs",
                name: "DEFAULT_ALPHA_0",
            },
        ),
        (
            "tab:config:companion",
            r"$\beta_0$ (prior pass pseudo-count)",
            Binding::Constant {
                source: "risk/challenge.rs",
                name: "DEFAULT_BETA_0",
            },
        ),
        (
            "tab:config:companion",
            r"$\gamma_{q,t}$ (pseudo-count time decay)",
            Binding::Constant {
                source: "risk/challenge.rs",
                name: "DEFAULT_GAMMA_QT",
            },
        ),
        (
            "tab:config:companion",
            r"Injection ceiling, per call",
            Binding::Constant {
                source: "risk/challenge.rs",
                name: "DEFAULT_INJECTION_CEILING",
            },
        ),
    ];

    let mut map = BTreeMap::new();
    for (table, parameter, binding) in ROWS {
        let key = format!("{table}|{parameter}");
        assert!(map.insert(key.clone(), *binding).is_none(), "one binding per row: {key}");
    }
    map
}

/// The declaration-surface parameters no configuration table governs, each with
/// the reason it is not tabulated. The projection requires the reason to be
/// there: a parameter nobody has accounted for is a finding, and this list is
/// where an account is given rather than where one is assumed.
const UNTABULATED: &[(&str, &str)] = &[
    (
        "AssayerConfig.instance_id",
        "an instance identifier for thread names and logs, not a parameter of the system's behaviour",
    ),
    (
        "AssayerConfig.persistence",
        "the checkpoint and journal directories and cadence, which the construction contract carries rather than this chapter",
    ),
    (
        "ConcordanceConfig.window_capacity",
        "concordance window sizes belong to the construction contract's parameters, not to the configuration chapter",
    ),
    (
        "ConcordanceConfig.recalibration_interval",
        "concordance recalibration cadence belongs to the construction contract's parameters",
    ),
    (
        "ConcordanceConfig.percentile",
        "the concordance threshold percentile belongs to the construction contract's parameters",
    ),
    (
        "ConvergenceThresholdConfig.platt_converged_delta_cal",
        "a convergence diagnostic that is reported and never enforced, carried by the construction contract",
    ),
    (
        "ConvergenceThresholdConfig.platt_converged_refit_count",
        "a convergence diagnostic that is reported and never enforced, carried by the construction contract",
    ),
    (
        "BlendStatisticsConfig.window_capacity",
        "the blend tracking window is a health-surface internal, tabulated nowhere in this chapter",
    ),
    (
        "BlendStatisticsConfig.publish_interval",
        "the blend publication cadence is a health-surface internal, tabulated nowhere in this chapter",
    ),
    (
        "HibernationConfig.expiry_days",
        "the hibernation archive's wall-clock lifetime, which the construction contract's parameter table carries rather than this chapter",
    ),
    (
        "InfrastructureConfig.command_channel_capacity",
        "host-set with no default, so the construction contract names it rather than a table of defaults",
    ),
    (
        "InfrastructureConfig.health_event_capacity",
        "the health event channel's depth is a construction-contract capacity",
    ),
    (
        "InfrastructureConfig.identity_observation_capacity",
        "the deferred identity queue's depth, which the concurrency table states as a shape rather than a number",
    ),
    (
        "InfrastructureConfig.identity_registration_timeout_secs",
        "a liveness bound on the registration acknowledgement, not a tuning parameter the chapter tabulates",
    ),
    (
        "StandardisationConfig.epsilon",
        "a division guard of the standardisation arithmetic, not a host-facing parameter",
    ),
    (
        "PlattConfig.golden_tolerance",
        "the golden-section search's own stopping tolerance, internal to the fitting algorithm",
    ),
    (
        "PlattConfig.max_golden_iterations",
        "the golden-section search's own iteration cap, internal to the fitting algorithm",
    ),
    (
        "ExtractionConfig.concordance_thresholds",
        "per-axis concordance thresholds, calibrated at runtime rather than defaulted by the chapter",
    ),
    (
        "LabelGuidanceParams.investigation_proxy",
        "a scoring-mode switch on the guidance call, not a tabulated quantity",
    ),
    (
        "ChannelPolicy.actions",
        "the declared action set, a structure the derivation reads rather than a value the chapter tabulates",
    ),
    (
        "RewardParameters.gamma_q_t",
        "the Companion's own decay carried on the reward record; its value is tabulated at tab:config:companion",
    ),
];

// ═══════════════════════════════════════════════════════════════════════════════
// Domains
// ═══════════════════════════════════════════════════════════════════════════════

/// One side of a tabulated constraint.
#[derive(Clone, Copy, Debug)]
enum Bound {
    Above(f64),
    AtLeast(f64),
    Below(f64),
    AtMost(f64),
}

impl Bound {
    fn holds(self, value: f64) -> bool {
        match self {
            Self::Above(limit) => value > limit,
            Self::AtLeast(limit) => value >= limit,
            Self::Below(limit) => value < limit,
            Self::AtMost(limit) => value <= limit,
        }
    }

    fn describe(self) -> String {
        match self {
            Self::Above(limit) => format!("> {limit}"),
            Self::AtLeast(limit) => format!(">= {limit}"),
            Self::Below(limit) => format!("< {limit}"),
            Self::AtMost(limit) => format!("<= {limit}"),
        }
    }
}

/// A tabulated constraint, as the row states it.
#[derive(Clone, Debug, Default)]
struct Domain {
    bounds: Vec<Bound>,
    alternatives: Vec<String>,
    boolean: bool,
}

/// Resolves a bound's operand: a number, or another parameter's symbol read
/// through the table that carries its value.
fn read_bound_operand(text: &str, symbols: &BTreeMap<String, f64>) -> Option<f64> {
    let text = text.trim();
    if let Some(value) = read_power_of_ten(text) {
        return Some(value);
    }
    if let Some(value) = read_number(text) {
        return Some(value);
    }
    symbols.get(text).copied()
}

/// Reads the constraint column: an interval, a comparison, a pair of
/// alternatives, or the em dash a structural row carries instead of one.
fn read_domain(cell: &str, symbols: &BTreeMap<String, f64>) -> Domain {
    let mut domain = Domain::default();
    let trimmed = cell.trim();
    if trimmed == "boolean" {
        domain.boolean = true;
        return domain;
    }
    if !trimmed.contains('$') {
        if trimmed.contains(" or ") {
            for alternative in trimmed.split(" or ") {
                for part in alternative.split(',') {
                    let word = part.trim().trim_end_matches(',').trim();
                    if !word.is_empty() {
                        domain.alternatives.push(word.to_lowercase());
                    }
                }
            }
        }
        return domain;
    }

    for group in trimmed.split('$').skip(1).step_by(2) {
        let group = group.trim();
        let opens_interval = group.starts_with('(') || group.starts_with('[');
        let closes_interval = group.ends_with(')') || group.ends_with(']');
        if opens_interval && closes_interval && group.contains(',') {
            let inner = &group[1..group.len() - 1];
            let Some((low, high)) = inner.split_once(',') else { continue };
            if let Some(value) = read_bound_operand(low, symbols) {
                domain.bounds.push(if group.starts_with('[') {
                    Bound::AtLeast(value)
                } else {
                    Bound::Above(value)
                });
            }
            if let Some(value) = read_bound_operand(high, symbols) {
                domain.bounds.push(if group.ends_with(']') {
                    Bound::AtMost(value)
                } else {
                    Bound::Below(value)
                });
            }
            continue;
        }
        let (make, operand) = if let Some(rest) = group.strip_prefix(r"\geq") {
            (Bound::AtLeast as fn(f64) -> Bound, rest)
        } else if let Some(rest) = group.strip_prefix(r"\leq") {
            (Bound::AtMost as fn(f64) -> Bound, rest)
        } else if let Some(rest) = group.strip_prefix('>') {
            (Bound::Above as fn(f64) -> Bound, rest)
        } else if let Some(rest) = group.strip_prefix('<') {
            (Bound::Below as fn(f64) -> Bound, rest)
        } else {
            continue;
        };
        if let Some(value) = read_bound_operand(operand, symbols) {
            domain.bounds.push(make(value));
        }
    }
    domain
}

// ═══════════════════════════════════════════════════════════════════════════════
// The oracle
// ═══════════════════════════════════════════════════════════════════════════════

/// What a projection can find wrong. The classes are kept apart because the
/// three drift fixtures are exactly the three a configuration surface suffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Finding {
    /// A table row no binding accounts for.
    UnaccountedRow,
    /// A binding naming a row no table carries.
    StaleBinding,
    /// A declaration-surface parameter no row and no stated reason accounts for.
    UnexplainedParameter,
    /// A binding naming a declaration the package no longer carries.
    MissingSurface,
    /// A shipped default that is not the tabulated one.
    Value,
    /// A shipped default outside its tabulated domain.
    Domain,
    /// A declared departure the package does not actually make.
    Departure,
}

/// One finding, naming the subject it is about.
#[derive(Clone, Debug)]
struct Mismatch {
    finding: Finding,
    subject: String,
    detail: String,
}

/// The projection: every row against its binding, every parameter against the
/// rows and reasons that account for it.
fn project(
    rows: &[SpecRow],
    bindings: &BTreeMap<String, Binding>,
    parameters: &BTreeMap<String, Value>,
    census: &BTreeSet<String>,
    shipped: &BTreeMap<String, Value>,
) -> Vec<Mismatch> {
    let mut mismatches = Vec::new();
    let symbols = symbol_values(rows);
    let untabulated: BTreeMap<_, _> = UNTABULATED.iter().copied().collect();

    let mut row_keys = BTreeSet::new();
    for row in rows {
        let key = row.key();
        row_keys.insert(key.clone());
        let Some(binding) = bindings.get(&key) else {
            mismatches.push(Mismatch {
                finding: Finding::UnaccountedRow,
                subject: key,
                detail: "no binding projects this table row onto the package".to_owned(),
            });
            continue;
        };
        check_row(row, *binding, shipped, &symbols, &mut mismatches);
    }

    for key in bindings.keys() {
        if !row_keys.contains(key) {
            mismatches.push(Mismatch {
                finding: Finding::StaleBinding,
                subject: key.clone(),
                detail: "the binding names a row no configuration table carries".to_owned(),
            });
        }
    }

    let claimed: BTreeSet<_> = bindings.values().filter_map(|binding| binding.claimed_parameter()).collect();
    for key in census {
        if claimed.contains(key) {
            continue;
        }
        match untabulated.get(key.as_str()) {
            Some(reason) if !reason.is_empty() => {}
            _ => mismatches.push(Mismatch {
                finding: Finding::UnexplainedParameter,
                subject: key.clone(),
                detail: "the package declares a parameter no table row and no stated reason accounts for".to_owned(),
            }),
        }
    }

    for (key, binding) in bindings {
        let Some(claimed_key) = binding.claimed_parameter() else {
            continue;
        };
        if !parameters.contains_key(&claimed_key) {
            mismatches.push(Mismatch {
                finding: Finding::MissingSurface,
                subject: key.clone(),
                detail: format!("the binding names {claimed_key}, which the package no longer declares"),
            });
        }
    }

    mismatches
}

/// One row against its binding: the tabulated value against the shipped one,
/// and the shipped one against the tabulated domain.
fn check_row(
    row: &SpecRow,
    binding: Binding,
    shipped: &BTreeMap<String, Value>,
    symbols: &BTreeMap<String, f64>,
    mismatches: &mut Vec<Mismatch>,
) {
    let Some(shipped_key) = binding.shipped_key() else { return };
    let Some(live) = shipped.get(&shipped_key) else { return };

    let scale = match binding {
        Binding::Field { scale, .. } => scale,
        _ => 1.0,
    };
    let mut tabulated = read_spec_value(&row.default_cell);
    if let Value::Symbol(symbol) = &tabulated
        && let Some(value) = symbols.get(symbol)
    {
        tabulated = Value::Number(*value);
    }

    if let Binding::Departure { note, .. } = binding {
        match (&tabulated, live) {
            (Value::Number(specified), Value::Number(actual)) if *specified * scale != *actual => {}
            _ => mismatches.push(Mismatch {
                finding: Finding::Departure,
                subject: row.key(),
                detail: format!("the declared departure ({note}) is not there: the package ships {live:?}"),
            }),
        }
        return;
    }

    match (&tabulated, live) {
        (Value::Number(specified), Value::Number(actual)) => {
            if *specified * scale != *actual {
                mismatches.push(Mismatch {
                    finding: Finding::Value,
                    subject: row.key(),
                    detail: format!(
                        "tabulated {specified} (scaled {}) against shipped {actual}",
                        specified * scale
                    ),
                });
            }
        }
        (Value::Boolean(specified), Value::Boolean(actual)) => {
            if specified != actual {
                mismatches.push(Mismatch {
                    finding: Finding::Value,
                    subject: row.key(),
                    detail: format!("tabulated {specified} against shipped {actual}"),
                });
            }
        }
        (Value::Text(specified), Value::Text(actual)) => {
            if Value::word(specified) != Value::word(actual) {
                mismatches.push(Mismatch {
                    finding: Finding::Value,
                    subject: row.key(),
                    detail: format!("tabulated {specified} against shipped {actual}"),
                });
            }
        }
        (specified, actual) => mismatches.push(Mismatch {
            finding: Finding::Value,
            subject: row.key(),
            detail: format!("tabulated {specified:?} and shipped {actual:?} are not the same kind of value"),
        }),
    }

    let domain = read_domain(&row.constraint_cell, symbols);
    match live {
        Value::Number(actual) => {
            let in_table_units = actual / scale;
            for bound in &domain.bounds {
                if !bound.holds(in_table_units) {
                    mismatches.push(Mismatch {
                        finding: Finding::Domain,
                        subject: row.key(),
                        detail: format!("shipped {in_table_units} leaves the tabulated domain {}", bound.describe()),
                    });
                }
            }
        }
        Value::Text(actual) => {
            if !domain.alternatives.is_empty() && !domain.alternatives.contains(&Value::word(actual)) {
                mismatches.push(Mismatch {
                    finding: Finding::Domain,
                    subject: row.key(),
                    detail: format!(
                        "shipped {actual} is none of the tabulated alternatives {:?}",
                        domain.alternatives
                    ),
                });
            }
        }
        Value::Boolean(_) | Value::Delegated(_) | Value::Symbol(_) => {}
    }
}

/// Everything a binding can look a shipped value up under: the declaration
/// surfaces' parameters, plus the named constants and the one derived internal
/// field the tables reach into.
fn shipped_values(parameters: &BTreeMap<String, Value>, bindings: &BTreeMap<String, Binding>) -> BTreeMap<String, Value> {
    let mut shipped = parameters.clone();
    for binding in bindings.values() {
        match *binding {
            Binding::Constant {
                source: name,
                name: constant,
            } => {
                let expression = constant_expression(name, constant);
                let value = read_package_value(name, &expression);
                shipped.insert(format!("{name}::{constant}"), value);
            }
            Binding::Internal {
                source: name, ty, field, ..
            } => {
                let fields = default_body_fields(name, ty);
                let expression = fields
                    .get(field)
                    .unwrap_or_else(|| panic!("{ty} declares a default for {field}"));
                let value = read_package_value(name, expression);
                shipped.insert(format!("{ty}.{field}"), value);
            }
            _ => {}
        }
    }
    shipped
}

/// Renders findings for a refusal message.
fn describe(mismatches: &[Mismatch]) -> String {
    mismatches
        .iter()
        .map(|mismatch| format!("[{:?}] {}: {}", mismatch.finding, mismatch.subject, mismatch.detail))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether any finding of a class names a subject, in the row it is about or in
/// the declaration its detail quotes.
fn found(mismatches: &[Mismatch], finding: Finding, subject: &str) -> bool {
    mismatches
        .iter()
        .any(|mismatch| mismatch.finding == finding && (mismatch.subject.contains(subject) || mismatch.detail.contains(subject)))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Every row of the specification's sixteen configuration tables meets a live
/// package declaration, and every parameter those declarations carry meets a
/// row or a stated reason, with neither side written down: the rows are read
/// from the chapter's own source and the parameters from the `Default` bodies
/// the package declares, corroborated by serde's own view of the top-level
/// configuration. Each bound row is then checked twice — its tabulated value
/// against the shipped one and the shipped one against the tabulated domain —
/// so a default that moved and a default that left its constraint are separate
/// findings rather than one. A hand count of this agreement is what the chapter
/// says it will not keep (´chap:spec:configuration´), and a test holding its own
/// copy of either side would keep passing through exactly the edit it exists to
/// catch.
///
/// ´claim:config:every-configuration-table-row-projects-onto-a-live-package-declaration´
/// ´test:crate:configuration-tables-project-onto-the-live-declarations´
#[test]
fn configuration_tables_project_onto_the_live_declarations() {
    let rows = spec_rows();
    let (parameters, delegations) = package_declarations();
    let bindings = bindings();
    let census: BTreeSet<_> = parameters.keys().cloned().collect();
    let shipped = shipped_values(&parameters, &bindings);

    // Non-vacuity, first: both discoveries found something, and they found it
    // across every table and every declaration surface rather than in one
    // corner each.
    let tables: BTreeSet<_> = rows.iter().map(|row| row.table.clone()).collect();
    assert_eq!(
        tables.len(),
        16,
        "the chapter's sixteen configuration tables were discovered: {tables:?}"
    );
    assert!(
        rows.len() >= 90,
        "the tables carry their roughly ninety rows, found {}",
        rows.len()
    );
    let surfaces: BTreeSet<_> = census
        .iter()
        .filter_map(|key| key.split('.').next().map(str::to_owned))
        .collect();
    assert_eq!(
        surfaces.len(),
        ROOTS.len(),
        "every declaration surface contributed parameters: {surfaces:?}"
    );

    // Non-vacuity, second: the two sides are genuinely different surfaces
    // rather than one surface read twice. Rows exist that no field carries, and
    // fields exist that no row tabulates; were either containment total, the
    // correspondence below would hold for a reason that has nothing to do with
    // the configuration chapter.
    let claimed: BTreeSet<_> = bindings.values().filter_map(|binding| binding.claimed_parameter()).collect();
    assert!(
        bindings.values().any(|binding| binding.claimed_parameter().is_none()),
        "rows exist that no declaration-surface field carries"
    );
    assert!(
        census.difference(&claimed).next().is_some(),
        "the package declares parameters no table row carries"
    );

    // Every delegation between declaration surfaces reaches a surface this
    // projection knows about, so a sub-configuration added to the Core surface
    // cannot slip past the census by being a struct rather than a scalar.
    let known: BTreeSet<_> = ROOTS.iter().map(|(ty, _)| (*ty).to_owned()).collect();
    let untabulated: BTreeSet<_> = UNTABULATED.iter().map(|(key, _)| (*key).to_owned()).collect();
    for (key, target) in &delegations {
        assert!(
            known.contains(target) || untabulated.contains(key),
            "the delegation {key} reaches {target}, which is neither a declaration surface nor accounted for"
        );
    }

    // Every stated reason is actually stated, and every reason names a
    // parameter the package declares — a reason for a field that has gone away
    // is as stale as a binding for a row that has gone away.
    for (key, reason) in UNTABULATED {
        assert!(!reason.is_empty(), "the reason for {key} is stated");
        assert!(
            census.contains(*key),
            "the reason for {key} names a parameter the package declares"
        );
    }

    // Every row that reaches no field of a declaration surface says why in its
    // own binding, so the rows the package does not expose are accounted for
    // rather than merely unbound.
    for (key, binding) in &bindings {
        if let Some(account) = binding.account() {
            assert!(!account.is_empty(), "the row {key} states the account its binding gives");
        }
    }

    // The obligation itself, against the live pair.
    let mismatches = project(&rows, &bindings, &parameters, &census, &shipped);
    assert!(
        mismatches.is_empty(),
        "the configuration chapter and the package declarations disagree:\n{}",
        describe(&mismatches)
    );

    // Corroboration: where a declaration surface derives serde, serde's own
    // view of the top-level configuration agrees with what the `Default` body
    // was scraped for. The scrape is the discovery; this is the check that the
    // discovery reads the same structure the compiler does.
    #[cfg(feature = "serde")]
    {
        let live = crate::config::types::AssayerConfig::default();
        let json = serde_json::to_value(&live).expect("the configuration serialises");
        let object = json.as_object().expect("the configuration serialises as an object");
        let reflected: BTreeSet<_> = object.keys().cloned().collect();
        let scraped: BTreeSet<_> = default_body_fields("config/types.rs", "AssayerConfig")
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            reflected, scraped,
            "serde's view of the configuration and the scraped `Default` body name the same fields"
        );
    }
}

/// The projection refuses each of the three ways configuration can drift, one
/// fabricated fixture at a time and every bound row load-bearing: a shipped
/// default moved off its tabulated value is refused and named, a shipped
/// default pushed outside its tabulated domain is refused as a domain finding
/// rather than as a value one, and exposure drift is refused in all three
/// directions it takes — a package parameter no row and no stated reason
/// accounts for, a declared surface the package no longer carries, and a table
/// row no binding accounts for. A guardrail that only checked the live pair
/// would pass on the day its own oracle stopped working.
///
/// ´claim:config:the-projection-refuses-value-domain-and-exposure-drift´
/// ´test:crate:configuration-projection-refuses-value-domain-and-exposure-drift´
#[test]
fn configuration_projection_refuses_value_domain_and_exposure_drift() {
    let rows = spec_rows();
    let (parameters, _) = package_declarations();
    let bindings = bindings();
    let census: BTreeSet<_> = parameters.keys().cloned().collect();
    let shipped = shipped_values(&parameters, &bindings);
    let symbols = symbol_values(&rows);

    // Value drift, once for every bound row. Each is individually load-bearing:
    // moving its shipped value makes this same oracle refuse and name the row,
    // so no bound row is carried along unchecked by the others.
    let mut moved_rows = 0_usize;
    for row in &rows {
        let Some(binding) = bindings.get(&row.key()) else { continue };
        let Some(key) = binding.shipped_key() else { continue };
        let Some(live) = shipped.get(&key) else { continue };
        let drifted = match live {
            Value::Number(value) => Value::Number(value + 1.0),
            Value::Boolean(value) => Value::Boolean(!value),
            Value::Text(value) => Value::Text(format!("{value}-moved")),
            Value::Delegated(_) | Value::Symbol(_) => continue,
        };
        let mut fixture = shipped.clone();
        fixture.insert(key, drifted);
        let mismatches = project(&rows, &bindings, &parameters, &census, &fixture);
        assert!(
            found(&mismatches, Finding::Value, &row.key()),
            "a shipped default moved off its tabulated value is refused and named: {}",
            row.key()
        );
        moved_rows += 1;
    }
    assert!(
        moved_rows >= 60,
        "the value fixture covered the bound rows, {moved_rows} of them"
    );

    // Domain drift, once for every bound row whose constraint has a numeric
    // bound. Leaving the domain is reported as a domain finding in its own
    // right, so a value that stayed inside its constraint and a value that left
    // it are never the same report.
    let mut left_domain = 0_usize;
    for row in &rows {
        let Some(binding) = bindings.get(&row.key()) else { continue };
        let Some(key) = binding.shipped_key() else { continue };
        if !matches!(shipped.get(&key), Some(Value::Number(_))) {
            continue;
        }
        let domain = read_domain(&row.constraint_cell, &symbols);
        let Some(bound) = domain.bounds.first().copied() else {
            continue;
        };
        let scale = match *binding {
            Binding::Field { scale, .. } => scale,
            _ => 1.0,
        };
        let outside = match bound {
            Bound::Above(limit) | Bound::AtLeast(limit) => limit - 1.0,
            Bound::Below(limit) | Bound::AtMost(limit) => limit + 1.0,
        };
        let mut fixture = shipped.clone();
        fixture.insert(key, Value::Number(outside * scale));
        let mismatches = project(&rows, &bindings, &parameters, &census, &fixture);
        assert!(
            found(&mismatches, Finding::Domain, &row.key()),
            "a shipped default outside its tabulated domain is refused as a domain finding: {}",
            row.key()
        );
        left_domain += 1;
    }
    assert!(
        left_domain >= 55,
        "the domain fixture covered the bounded rows, {left_domain} of them"
    );

    // Exposure drift, first direction: the package grows a parameter no row and
    // no stated reason accounts for.
    let fabricated = "ModelConfig.no_such_parameter";
    assert!(!census.contains(fabricated), "the fabricated parameter is not declared");
    let mut gained = parameters.clone();
    gained.insert(fabricated.to_owned(), Value::Number(1.0));
    let gained_census: BTreeSet<_> = gained.keys().cloned().collect();
    let mismatches = project(&rows, &bindings, &gained, &gained_census, &shipped);
    assert!(
        found(&mismatches, Finding::UnexplainedParameter, fabricated),
        "an unexplained package parameter is refused and named:\n{}",
        describe(&mismatches)
    );

    // Exposure drift, second direction: a declared surface goes away under a
    // binding that still names it. Every bound field is put under this in turn,
    // so no binding names a surface whose disappearance would pass unnoticed.
    let mut surfaces_lost = 0_usize;
    for binding in bindings.values() {
        let Some(claimed) = binding.claimed_parameter() else {
            continue;
        };
        let mut lost = parameters.clone();
        assert!(lost.remove(&claimed).is_some(), "the ablated parameter was declared");
        let lost_census: BTreeSet<_> = lost.keys().cloned().collect();
        let mismatches = project(&rows, &bindings, &lost, &lost_census, &shipped);
        assert!(
            found(&mismatches, Finding::MissingSurface, &claimed),
            "a binding whose declaration went away is refused and names it: {claimed}"
        );
        surfaces_lost += 1;
    }
    assert!(
        surfaces_lost >= 60,
        "every bound field was ablated in turn, {surfaces_lost} of them"
    );

    // Exposure drift, third direction: a table gains a row no binding accounts
    // for, and a binding is left naming a row no table carries.
    let mut grown = rows.clone();
    grown.push(SpecRow {
        table: "tab:config:risk-model".to_owned(),
        parameter: "$X$ (a parameter no binding accounts for)".to_owned(),
        default_cell: "1".to_owned(),
        constraint_cell: r"$> 0$".to_owned(),
    });
    let mismatches = project(&grown, &bindings, &parameters, &census, &shipped);
    assert!(
        found(&mismatches, Finding::UnaccountedRow, "a parameter no binding accounts for"),
        "an unaccounted table row is refused and named:\n{}",
        describe(&mismatches)
    );

    let mut stale = bindings;
    stale.insert(
        "tab:config:risk-model|$Y$ (a row no table carries)".to_owned(),
        Binding::field("ModelConfig", "lambda_prior"),
    );
    let mismatches = project(&rows, &stale, &parameters, &census, &shipped);
    assert!(
        found(&mismatches, Finding::StaleBinding, "a row no table carries"),
        "a binding for a row no table carries is refused and named:\n{}",
        describe(&mismatches)
    );
}

/// A departure from a tabulated value is declared rather than exempted: the
/// projection accepts a row whose package declaration deliberately differs from
/// the specification only while that difference is actually there, and refuses
/// the declaration the moment the package agrees with the table again. The
/// chain-length normaliser is the row the chapter documents a departure for
/// (´tab:config:extraction´), and against the live package the departure is
/// absent — the shipped divisor is the tabulated one at both construction sites
/// — so the row binds as an ordinary agreement and the mechanism is exercised
/// against a fabricated package instead. An exemption comment would have gone
/// on excusing a difference that had already been repaired.
///
/// ´claim:config:a-declared-departure-is-honoured-only-while-the-package-actually-departs´
/// ´test:crate:configuration-projection-accounts-for-a-declared-departure´
#[test]
fn configuration_projection_accounts_for_a_declared_departure() {
    let rows = spec_rows();
    let (parameters, _) = package_declarations();
    let bindings = bindings();
    let census: BTreeSet<_> = parameters.keys().cloned().collect();
    let shipped = shipped_values(&parameters, &bindings);

    let key = r"tab:config:extraction|$D_\text{chain}$ (chain length normalisation)";
    assert!(bindings.contains_key(key), "the chain normalisation row is bound");

    // The live package agrees with the table, so the row binds as an ordinary
    // agreement: the divisor the extraction configuration declares is the
    // tabulated one, and the departure the chapter records is not there.
    let live = shipped
        .get("ExtractionConfig.d_chain_norm")
        .expect("the extraction configuration declares the chain normalisation");
    assert_eq!(
        *live,
        Value::Number(16.0),
        "the shipped chain-length divisor is the tabulated one"
    );

    // Were the departure declared against this package, it would be refused —
    // a declaration is honoured only while the difference it declares exists.
    let mut declared = bindings.clone();
    declared.insert(
        key.to_owned(),
        Binding::Departure {
            root: "ExtractionConfig",
            field: "d_chain_norm",
            note: "ships at 128 rather than at 16",
        },
    );
    let mismatches = project(&rows, &declared, &parameters, &census, &shipped);
    assert!(
        found(&mismatches, Finding::Departure, "chain length normalisation"),
        "a declared departure the package does not make is refused:\n{}",
        describe(&mismatches)
    );

    // And against a package that does depart, the same declaration is accepted
    // while the plain binding is refused — so the mechanism distinguishes a
    // documented difference from an undocumented one rather than excusing both.
    let mut departing = shipped.clone();
    departing.insert("ExtractionConfig.d_chain_norm".to_owned(), Value::Number(128.0));
    let honoured = project(&rows, &declared, &parameters, &census, &departing);
    assert!(
        honoured.is_empty(),
        "a declared departure the package does make is accepted:\n{}",
        describe(&honoured)
    );
    let undeclared = project(&rows, &bindings, &parameters, &census, &departing);
    assert!(
        found(&undeclared, Finding::Value, "chain length normalisation"),
        "the same difference without a declaration is refused as value drift:\n{}",
        describe(&undeclared)
    );
}
