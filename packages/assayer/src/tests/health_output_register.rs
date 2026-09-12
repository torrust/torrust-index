// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The generated whole-report health-field register.
//!
//! The projection reads the fields of `SystemHealthReport` and its sole
//! production constructor. A declared field enters the document only when that
//! constructor gives it one non-placeholder initializer. The declaration
//! supplies the field name, type, and description; the constructor supplies the
//! producer identity. Nothing is transcribed into a second inventory.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`health_output_register_matches_generated_bytes`] | audit | The whole-report field table is exactly the projection of the shipped `SystemHealthReport` definition and its production initializer; setting the explicit update switch rewrites that same region, and an ordinary test run only compares bytes. |
//! | [`health_output_register_refuses_a_mutated_row`] | audit | A hand edit inside the generated health-field table is staleness: mutating one emitted field name makes the same exact-byte check used on the committed chapter refuse the document. |
//! | [`health_output_register_fields_resolve_once_and_stops_cannot_ship`] | audit | Every emitted health field has exactly one entry in the sole production `SystemHealthReport` initializer, while a literal-`None` placeholder cannot enter and a quantity absent from the shipped type supplies no candidate row. |

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::testing::{SourceTreeScanner, manifest_dir};

const REPORT_STRUCT_HEAD: &str = "pub struct SystemHealthReport {";
const REPORT_CONSTRUCTOR: &str = "SystemHealthReport {";
// The generated register lives with the dated conformance audit rather than in
// the specification: the specification states what the outputs are and the
// audit carries the evidence of what this build emits, and the two were
// separated so the specification could be read blind to implementation status.
// The projection follows the document; it had been left pointing at the
// chapter the table was moved out of, where it found no register to compare
// against and could report nothing at all.
const REGISTER_RELATIVE_PATH: &str = "docs/plans/conformance-audit.md";
const REGISTER_HEADER: &str = "| Field | Type | Definition | Producer |";
const REGISTER_DELIMITER: &str = "| --- | --- | --- | --- |";
const PRODUCER_CELL: &str =
    "`Assayer::full_health_report` (`test:crate:health-output-register-fields-resolve-once-and-stops-cannot-ship`)";
const UPDATE_SWITCH: &str = "ASSAYER_UPDATE_HEALTH_OUTPUT_REGISTER";

/// Whole-report quantities that may not appear as a shipped top-level field.
///
/// The legacy three-component encoding formula is retired as superseded design
/// history (´entry:health:encoding-effectiveness´). Two entries have left this
/// list because they are delivered rather than STOPped, and each is delivered
/// under its own name rather than the one it was STOPped under. The Ledger's
/// realised value and its attenuation-limited pair sit on the Ledger's own
/// section of the report (´entry:memory:materiality´). Feedback latency ships
/// as `cross_layer`, carrying the four-stage decomposition
/// (´entry:health:cross-layer-latency´); the bare name stays out because the
/// undecomposed total is not what the report offers, and a field spelled that
/// way would promise a complete quantity where the first stage is a lower
/// bound (´def:monitoring:feedback-latency´).
const STOPPED_FIELD_NAMES: &[&str] = &["feedback_latency", "encoding_effectiveness"];

#[derive(Clone, Debug, PartialEq, Eq)]
struct FieldDefinition {
    name: String,
    type_name: String,
    description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Producer {
    expression: String,
    source: PathBuf,
    line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProjectedField {
    definition: FieldDefinition,
    producer: Producer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HealthOutputProjection {
    fields: Vec<ProjectedField>,
    excluded_placeholders: BTreeSet<String>,
    declared_fields: BTreeSet<String>,
}

fn read_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("could not read {}: {error}", path.display()))
}

fn report_fields(source: &str) -> Result<Vec<FieldDefinition>, String> {
    let mut lines = source.lines();
    let Some(head) = lines.find(|line| line.trim() == REPORT_STRUCT_HEAD) else {
        return Err(format!("could not find `{REPORT_STRUCT_HEAD}`"));
    };
    if head.trim() != REPORT_STRUCT_HEAD {
        return Err("the report declaration head was not exact".to_owned());
    }

    let mut definitions = Vec::new();
    let mut documentation = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }

        if let Some(text) = trimmed.strip_prefix("///") {
            let text = text.trim();
            if !text.is_empty() {
                documentation.push(text);
            }
            continue;
        }

        let Some(field) = trimmed.strip_prefix("pub ") else {
            continue;
        };
        let Some((name, type_name)) = field.trim_end_matches(',').split_once(':') else {
            return Err(format!("could not read report field declaration `{trimmed}`"));
        };
        if documentation.is_empty() {
            return Err(format!("report field `{name}` has no definition documentation"));
        }

        definitions.push(FieldDefinition {
            name: name.trim().to_owned(),
            type_name: type_name.trim().to_owned(),
            description: documentation.join(" "),
        });
        documentation.clear();
    }

    if definitions.is_empty() {
        return Err("the report declaration yielded no fields".to_owned());
    }

    Ok(definitions)
}

fn production_constructor_sites() -> Vec<(PathBuf, usize)> {
    let mut sites = Vec::new();

    SourceTreeScanner::new().skip_dir("tests").for_each_line(|visit| {
        let trimmed = visit.line.trim();
        if !visit.in_cfg_test && trimmed == REPORT_CONSTRUCTOR {
            sites.push((visit.path.to_path_buf(), visit.line_no));
        }
    });

    sites
}

fn braces(line: &str) -> (usize, usize) {
    line.chars().fold((0, 0), |(opening, closing), character| match character {
        '{' => (opening + 1, closing),
        '}' => (opening, closing + 1),
        _ => (opening, closing),
    })
}

fn constructor_producers(path: &Path, line_number: usize) -> Result<BTreeMap<String, Vec<Producer>>, String> {
    let source = read_source(path)?;
    let lines: Vec<&str> = source.lines().collect();
    let Some(head) = lines.get(line_number.saturating_sub(1)) else {
        return Err(format!("constructor location is outside {}", path.display()));
    };
    if head.trim() != REPORT_CONSTRUCTOR {
        return Err(format!("constructor moved from {}:{line_number}", path.display()));
    }

    let mut depth = 1_usize;
    let mut producers: BTreeMap<String, Vec<Producer>> = BTreeMap::new();

    for (offset, line) in lines.iter().enumerate().skip(line_number) {
        let trimmed = line.trim();
        if depth == 1 && trimmed == "}" {
            break;
        }

        if depth == 1 && !trimmed.is_empty() && !trimmed.starts_with("//") {
            let entry = trimmed.trim_end_matches(',');
            let (name, expression) = entry
                .split_once(':')
                .map_or((entry, entry), |(name, expression)| (name.trim(), expression.trim()));
            producers.entry(name.to_owned()).or_default().push(Producer {
                expression: expression.to_owned(),
                source: path.to_path_buf(),
                line: offset + 1,
            });
        }

        let (opening, closing) = braces(line);
        depth = depth.saturating_add(opening).saturating_sub(closing);
    }

    Ok(producers)
}

fn health_output_projection() -> Result<HealthOutputProjection, String> {
    let root = manifest_dir();
    let definitions = report_fields(&read_source(&root.join("src/health/summary.rs"))?)?;
    let sites = production_constructor_sites();
    let [(constructor_path, constructor_line)] = sites.as_slice() else {
        return Err(format!(
            "expected one production `{REPORT_CONSTRUCTOR}` initializer, found {} at {sites:?}",
            sites.len()
        ));
    };
    let producers = constructor_producers(constructor_path, *constructor_line)?;
    let mut fields = Vec::new();
    let mut excluded_placeholders = BTreeSet::new();
    let mut declared_fields = BTreeSet::new();

    for definition in definitions {
        let _new = declared_fields.insert(definition.name.clone());
        let Some(field_producers) = producers.get(&definition.name) else {
            return Err(format!("report field `{}` has no production initializer", definition.name));
        };
        let [producer] = field_producers.as_slice() else {
            return Err(format!(
                "report field `{}` has {} production initializers: {field_producers:?}",
                definition.name,
                field_producers.len()
            ));
        };

        if producer.expression == "None" {
            let _new = excluded_placeholders.insert(definition.name);
            continue;
        }

        fields.push(ProjectedField {
            definition,
            producer: producer.clone(),
        });
    }

    Ok(HealthOutputProjection {
        fields,
        excluded_placeholders,
        declared_fields,
    })
}

fn escaped_cell(text: &str) -> String {
    text.replace('|', "\\|")
}

fn rendered_region(projection: &HealthOutputProjection) -> Vec<String> {
    let mut region = vec![REGISTER_HEADER.to_owned(), REGISTER_DELIMITER.to_owned()];

    for field in &projection.fields {
        region.push(format!(
            "| `{}` | `{}` | {} | {} |",
            field.definition.name,
            escaped_cell(&field.definition.type_name),
            escaped_cell(&field.definition.description),
            PRODUCER_CELL,
        ));
    }

    region
}

fn rewrite_register(document: &str, region: &[String]) -> Result<String, String> {
    let mut lines: Vec<String> = document.split('\n').map(str::to_owned).collect();
    let headers: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (line == REGISTER_HEADER).then_some(index))
        .collect();
    let [first] = headers.as_slice() else {
        return Err(format!("expected one generated register header, found {}", headers.len()));
    };
    let mut past_last = *first + 1;
    while lines.get(past_last).is_some_and(|line| line.starts_with('|')) {
        past_last += 1;
    }
    let _replaced: Vec<String> = lines.splice(*first..past_last, region.iter().cloned()).collect();

    Ok(lines.join("\n"))
}

fn generated_document() -> Result<(PathBuf, String, String), String> {
    let path = manifest_dir().join(REGISTER_RELATIVE_PATH);
    let committed = read_source(&path)?;
    let projection = health_output_projection()?;
    let regenerated = rewrite_register(&committed, &rendered_region(&projection))?;

    Ok((path, committed, regenerated))
}

fn check_exact_bytes(committed: &str, regenerated: &str) -> Result<(), String> {
    if committed == regenerated {
        Ok(())
    } else {
        Err("the generated health output register is stale".to_owned())
    }
}

/// The whole-report field table is exactly the projection of the shipped
/// `SystemHealthReport` definition and its production initializer; setting the
/// explicit update switch rewrites that same region, and an ordinary test run
/// only compares bytes.
///
/// ´claim:audit:the-health-output-register-is-an-exact-projection-of-the-shipped-report´
/// ´test:crate:health-output-register-matches-generated-bytes´
#[test]
fn health_output_register_matches_generated_bytes() {
    let generated = generated_document();
    assert!(generated.is_ok(), "the health output projection is readable: {generated:?}");
    let Ok((path, committed, regenerated)) = generated else {
        return;
    };

    if std::env::var_os(UPDATE_SWITCH).is_some() {
        let written = fs::write(&path, &regenerated);
        assert!(
            written.is_ok(),
            "the generated register can be written to {}: {written:?}",
            path.display()
        );
    } else {
        let checked = check_exact_bytes(&committed, &regenerated);
        assert!(
            checked.is_ok(),
            "the generated health output register is current: {checked:?}"
        );
    }
}

/// A hand edit inside the generated health-field table is staleness: mutating
/// one emitted field name makes the same exact-byte check used on the committed
/// chapter refuse the document.
///
/// ´claim:audit:a-hand-edit-in-the-health-output-register-is-staleness´
/// ´test:crate:health-output-register-refuses-a-mutated-row´
#[test]
fn health_output_register_refuses_a_mutated_row() {
    let generated = generated_document();
    assert!(generated.is_ok(), "the health output projection is readable: {generated:?}");
    let Ok((_path, committed, regenerated)) = generated else {
        return;
    };
    let mutated = regenerated.replacen("| `convergence` |", "| `hand_edited_convergence` |", 1);
    assert_ne!(mutated, regenerated, "the fixture mutates one generated row");
    let checked = check_exact_bytes(&mutated, &committed);
    assert_eq!(
        checked,
        Err("the generated health output register is stale".to_owned()),
        "the exact-byte check refuses the mutation"
    );
}

/// Every emitted health field has exactly one entry in the sole production
/// `SystemHealthReport` initializer, while a literal-`None` placeholder cannot
/// enter and a quantity absent from the shipped type supplies no candidate row.
///
/// ´claim:audit:each-generated-health-field-resolves-once-and-unimplemented-quantities-cannot-enter´
/// ´test:crate:health-output-register-fields-resolve-once-and-stops-cannot-ship´
#[test]
fn health_output_register_fields_resolve_once_and_stops_cannot_ship() {
    let projection = health_output_projection();
    assert!(
        projection.is_ok(),
        "every declared report field resolves once: {projection:?}"
    );
    let Ok(projection) = projection else {
        return;
    };
    assert!(
        projection.fields.iter().all(|field| {
            field.producer.source.ends_with("src/api/health.rs") && field.producer.line > 0 && field.producer.expression != "None"
        }),
        "every displayed producer is a concrete entry in the full-report initializer"
    );
    let emitted: BTreeSet<&str> = projection.fields.iter().map(|field| field.definition.name.as_str()).collect();

    assert!(
        !projection.excluded_placeholders.contains("cross_layer"),
        "the latency field is no longer a literal-None placeholder"
    );
    assert!(
        emitted.contains("cross_layer"),
        "the delivered latency decomposition is a shipped row"
    );

    for stopped in STOPPED_FIELD_NAMES {
        assert!(
            !projection.declared_fields.contains(*stopped),
            "the STOPped quantity `{stopped}` has no shipped field to project"
        );
        assert!(
            !emitted.contains(*stopped),
            "the STOPped quantity `{stopped}` cannot be emitted"
        );
    }
}
