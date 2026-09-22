// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Repository-hygiene scanning helpers.
//!
//! Source-audit tests (e.g. "no `fn stub_…` in production code",
//! "no `.powf(` outside `numerics.rs`") all share the same shape:
//! walk `src/`, open every `.rs` file, inspect each line. This module
//! factors that boilerplate into a small builder so the tests
//! themselves stay focused on the pattern they are looking for.
//!
//! # Example
//!
//! ```ignore
//! use crate::testing::SourceTreeScanner;
//!
//! let mut hits = Vec::new();
//! SourceTreeScanner::new()
//!     .skip_dir("tests")
//!     .for_each_line(|v| {
//!         if !v.in_cfg_test && v.line.trim_start().starts_with("fn stub_") {
//!             hits.push(format!("{}:{}", v.path.display(), v.line_no));
//!         }
//!     });
//! ```
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`a_missing_scan_root_fails_the_audit_naming_the_path`] | audit | A scan root that cannot be read is a failed audit, not an empty tree. |
//! | [`an_unreadable_source_file_fails_the_audit_naming_the_path`] | audit | cites (´claim:audit:a-source-audit-that-cannot-read-the-tree-fails-instead-of-reporting-it-clean´) |
//! | [`a_guarded_variant_does_not_hide_the_items_after_it`] | audit | A guarded enum variant covers the variant and stops there. |
//! | [`a_guarded_declaration_does_not_hide_the_rest_of_the_file`] | audit | cites (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´) |
//! | [`a_guarded_module_does_not_hide_the_items_after_it`] | audit | cites (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´) |
//! | [`nested_braces_inside_a_guarded_item_do_not_end_it_early`] | audit | cites (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´) |
//! | [`a_guarded_variant_no_longer_hides_the_maintenance_loop`] | audit | cites (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´) |

use std::fs;
use std::path::{Path, PathBuf};

/// The manifest directory of the checkout this test run is scanning.
///
/// A source audit asks a question about *the tree it is being run
/// against*, which is runtime information, not compile-time
/// information. Cargo sets `CARGO_MANIFEST_DIR` in the environment of
/// every `cargo test` execution and it names the invoking checkout, so
/// the runtime value is the one that answers the question. The
/// compile-time `env!` value names whichever checkout the test binary
/// happened to be *compiled* in, which is a different tree the moment
/// a build cache is shared between clones: the artifact is reused, the
/// baked path still points at the clone that produced it, and the walk
/// audits somebody else's source while reporting on ours.
///
/// Reading the variable at runtime is what keeps the binary correct
/// wherever it was compiled — the property a shared build cache needs,
/// since the whole point of sharing is that one artifact serves every
/// clone. The compile-time value survives only as the fallback for a
/// test binary invoked bare, outside cargo, where nothing else names
/// the tree.
#[must_use]
pub fn manifest_dir() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR").map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")), PathBuf::from)
}

/// One visited line inside the source tree.
pub struct LineVisit<'a> {
    /// Absolute path of the file being scanned.
    pub path: &'a Path,
    /// 1-based line number within `path`.
    pub line_no: usize,
    /// The line itself, without its trailing newline.
    pub line: &'a str,
    /// `true` when this line belongs to an item a `#[cfg(test)]`
    /// attribute covers.
    ///
    /// The attribute applies to the item that follows it and to nothing
    /// else, so the flag comes back down at that item's end: a guarded
    /// enum variant marks the variant, not the rest of the file. An
    /// inner `#![cfg(test)]` covers the whole file, because that is the
    /// item it is written on.
    pub in_cfg_test: bool,
}

/// Builder for walking the crate's `src/` tree line-by-line.
///
/// Created via [`SourceTreeScanner::new`], which roots the walk at
/// `$CARGO_MANIFEST_DIR/src` as resolved by [`manifest_dir`] — the
/// invoking checkout, read at runtime. Directories can be skipped; the
/// visitor is invoked for every remaining line in every `.rs` file.
pub struct SourceTreeScanner {
    root: PathBuf,
    skip_dirs: Vec<String>,
}

impl SourceTreeScanner {
    /// Create a scanner rooted at `$CARGO_MANIFEST_DIR/src`, resolved
    /// at runtime by [`manifest_dir`] so that the walk covers the
    /// checkout being tested rather than the one that compiled the
    /// binary.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: manifest_dir().join("src"),
            skip_dirs: Vec::new(),
        }
    }

    /// Skip any directory whose final component equals `name`.
    #[must_use]
    pub fn skip_dir(mut self, name: impl Into<String>) -> Self {
        self.skip_dirs.push(name.into());
        self
    }

    /// The directory the scan walks. Useful for assertions such as
    /// "`src/stubs.rs` stays deleted".
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Walk the tree and invoke `visit` once per line of every `.rs` file.
    ///
    /// A directory named by [`SourceTreeScanner::skip_dir`] is not
    /// descended into and is never opened, so its contents are outside
    /// the question the scan asks rather than an unreadable part of it.
    ///
    /// # Panics
    ///
    /// Panics, naming the path and the underlying error, if the scan
    /// root or any directory under it cannot be read, if a directory
    /// entry cannot be read, or if a `.rs` file cannot be read as UTF-8.
    ///
    /// An audit that answers a question about the source tree has not
    /// answered it if it could not read the tree. Every caller asserts
    /// that the violations it collected are empty, so a skipped file and
    /// a clean file are the same observation to them: swallowing the
    /// error reports a tree that was never examined as a tree with
    /// nothing wrong in it. Failing loudly is available here because the
    /// scanner runs only under `cfg(test)` or `test-support`, where a
    /// panic is how a test reports that it could not run.
    pub fn for_each_line<F>(&self, mut visit: F)
    where
        F: FnMut(LineVisit<'_>),
    {
        self.walk(&self.root, &mut visit);
    }

    fn walk<F>(&self, dir: &Path, visit: &mut F)
    where
        F: FnMut(LineVisit<'_>),
    {
        let entries =
            fs::read_dir(dir).unwrap_or_else(|error| panic!("source audit cannot read the directory {}: {error}", dir.display()));

        for entry in entries {
            let entry = entry.unwrap_or_else(|error| {
                panic!(
                    "source audit cannot read an entry of the directory {}: {error}",
                    dir.display()
                )
            });
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if self.skip_dirs.iter().any(|s| s == name) {
                    continue;
                }
                self.walk(&path, visit);
                continue;
            }

            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("source audit cannot read the file {}: {error}", path.display()));

            let mut scope = CfgTestScope::default();
            for (idx, line) in content.lines().enumerate() {
                let in_cfg_test = scope.covers(line);
                visit(LineVisit {
                    path: &path,
                    line_no: idx + 1,
                    line,
                    in_cfg_test,
                });
            }
        }
    }
}

/// Which lines of one file the `#[cfg(test)]` attributes in it cover.
///
/// A `#[cfg(test)]` applies to the item written after it. Treating it as a
/// switch that stays on for the rest of the file costs the audits every
/// production line that follows a guarded variant, a guarded match arm, or a
/// `mod tests;` declaration — and those lines are the ones an architecture
/// audit most wants to read, because a crate root declares its test module
/// early and its public surface after it.
#[derive(Default)]
struct CfgTestScope {
    view: CodeView,
    guarded: Option<GuardedItem>,
    whole_file: bool,
}

impl CfgTestScope {
    /// Take the next line of the file; `true` when a `#[cfg(test)]` covers it.
    fn covers(&mut self, line: &str) -> bool {
        let code = self.view.blank(line);
        if self.whole_file {
            return true;
        }

        if self.guarded.is_none() {
            let trimmed = code.trim_start();
            if trimmed.starts_with("#![") && trimmed.contains("cfg(test)]") {
                self.whole_file = true;
                return true;
            }
            if !(trimmed.starts_with("#[") && trimmed.contains("cfg(test)]")) {
                return false;
            }
            self.guarded = Some(GuardedItem::default());
        }

        if self.guarded.as_mut().is_some_and(|item| item.consume(&code)) {
            self.guarded = None;
        }
        true
    }
}

/// The extent of the item a `#[cfg(test)]` attribute guards.
///
/// A braced item — `mod`, `fn`, `impl`, a variant with fields — ends at the
/// close brace that brings the nesting back to zero. An item written without a
/// block — a `use`, a unit variant, a field, a match arm with a bare expression
/// — ends at its own terminator. Parentheses and square brackets are counted
/// beside the braces so that a terminator inside a multi-line signature or a
/// multi-line attribute is not mistaken for the item's own.
#[derive(Default)]
struct GuardedItem {
    parens: usize,
    brackets: usize,
    braces: usize,
    opened_block: bool,
}

impl GuardedItem {
    /// Consume one line of blanked code; `true` once the item has ended.
    fn consume(&mut self, code: &str) -> bool {
        let mut terminated = false;
        for character in code.chars() {
            match character {
                '(' => self.parens += 1,
                ')' => self.parens = self.parens.saturating_sub(1),
                '[' => self.brackets += 1,
                ']' => self.brackets = self.brackets.saturating_sub(1),
                '{' => {
                    self.braces += 1;
                    self.opened_block = true;
                }
                '}' => self.braces = self.braces.saturating_sub(1),
                ';' | ',' if !self.opened_block && self.balanced() => terminated = true,
                _ => {}
            }
        }

        terminated || (self.opened_block && self.balanced())
    }

    /// Whether every bracket opened on the item so far has been closed.
    const fn balanced(&self) -> bool {
        self.parens == 0 && self.brackets == 0 && self.braces == 0
    }
}

/// The string-literal and block-comment state carried between lines.
///
/// The scan reads a file line by line, but neither a literal nor a block
/// comment respects a line ending. Carrying the state lets each line be
/// reduced to its code, which is what makes the brace counting above answer a
/// question about items rather than about punctuation: `"{}"` in a format
/// string and `// }` in a comment are not braces the compiler sees.
#[derive(Default)]
struct CodeView {
    block_comment_depth: usize,
    literal: Option<Literal>,
}

/// The kind of string literal a scan is currently inside.
#[derive(Clone, Copy)]
enum Literal {
    /// A `"…"` literal, closed by the next unescaped quote.
    Quoted,
    /// A raw literal, closed by a quote followed by `hashes` hashes.
    Raw { hashes: usize },
}

impl CodeView {
    /// Return `line` with every comment and literal replaced by spaces.
    ///
    /// Column positions are preserved, so the result trims and matches exactly
    /// where the original line would have.
    fn blank(&mut self, line: &str) -> String {
        let source: Vec<_> = line.chars().collect();
        let mut code = String::with_capacity(line.len());
        let mut index = 0;

        while index < source.len() {
            index = match (self.block_comment_depth, self.literal) {
                (depth, _) if depth > 0 => self.step_block_comment(&source, index, &mut code),
                (_, Some(literal)) => self.step_literal(literal, &source, index, &mut code),
                _ => self.step_code(&source, index, &mut code),
            };
        }

        code
    }

    /// Advance one step inside a block comment, which Rust allows to nest.
    fn step_block_comment(&mut self, source: &[char], index: usize, code: &mut String) -> usize {
        if pair_at(source, index, '*', '/') {
            self.block_comment_depth -= 1;
            code.push_str("  ");
            return index + 2;
        }
        if pair_at(source, index, '/', '*') {
            self.block_comment_depth += 1;
            code.push_str("  ");
            return index + 2;
        }
        code.push(' ');
        index + 1
    }

    /// Advance one step inside a string literal.
    fn step_literal(&mut self, literal: Literal, source: &[char], index: usize, code: &mut String) -> usize {
        if let Literal::Raw { hashes } = literal {
            if source[index] == '"' && closes_raw(source, index + 1, hashes) {
                self.literal = None;
                push_spaces(code, hashes + 1);
                return index + hashes + 1;
            }
            code.push(' ');
            return index + 1;
        }

        if source[index] == '\\' {
            let width = if index + 1 < source.len() { 2 } else { 1 };
            push_spaces(code, width);
            return index + width;
        }
        if source[index] == '"' {
            self.literal = None;
        }
        code.push(' ');
        index + 1
    }

    /// Advance one step through ordinary code, opening a comment or literal.
    fn step_code(&mut self, source: &[char], index: usize, code: &mut String) -> usize {
        if pair_at(source, index, '/', '/') {
            push_spaces(code, source.len() - index);
            return source.len();
        }
        if pair_at(source, index, '/', '*') {
            self.block_comment_depth = 1;
            code.push_str("  ");
            return index + 2;
        }
        if let Some((width, hashes)) = raw_literal_opening(source, index) {
            self.literal = Some(Literal::Raw { hashes });
            push_spaces(code, width);
            return index + width;
        }
        if let Some(width) = quoted_literal_opening(source, index) {
            self.literal = Some(Literal::Quoted);
            push_spaces(code, width);
            return index + width;
        }
        if let Some(width) = character_literal_width(source, index) {
            push_spaces(code, width);
            return index + width;
        }
        code.push(source[index]);
        index + 1
    }
}

/// Whether `source` holds `first` then `second` at `index`.
fn pair_at(source: &[char], index: usize, first: char, second: char) -> bool {
    source.get(index) == Some(&first) && source.get(index + 1) == Some(&second)
}

/// Append `count` spaces, standing in for the characters they replace.
fn push_spaces(code: &mut String, count: usize) {
    code.extend(std::iter::repeat_n(' ', count));
}

/// Whether a character can appear inside an identifier.
fn is_identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// The width of a raw-literal opening at `index`, with its hash count.
///
/// The `r` is only a prefix when nothing is attached to its left; otherwise it
/// is the last letter of an identifier that happens to precede a quote.
fn raw_literal_opening(source: &[char], index: usize) -> Option<(usize, usize)> {
    if index > 0 && is_identifier_char(source[index - 1]) {
        return None;
    }

    let mut cursor = index;
    if source.get(cursor) == Some(&'b') {
        cursor += 1;
    }
    if source.get(cursor) != Some(&'r') {
        return None;
    }
    cursor += 1;

    let hashes_start = cursor;
    while source.get(cursor) == Some(&'#') {
        cursor += 1;
    }
    if source.get(cursor) != Some(&'"') {
        return None;
    }

    Some((cursor + 1 - index, cursor - hashes_start))
}

/// The width of a `"` or `b"` literal opening at `index`.
fn quoted_literal_opening(source: &[char], index: usize) -> Option<usize> {
    if source.get(index) == Some(&'"') {
        return Some(1);
    }
    let byte_string = source.get(index) == Some(&'b')
        && source.get(index + 1) == Some(&'"')
        && !(index > 0 && is_identifier_char(source[index - 1]));
    byte_string.then_some(2)
}

/// The width of a character literal at `index`, or `None` for a lifetime.
///
/// A quote opens a lifetime far more often than a literal in this tree, so the
/// two are told apart by shape: `'\n'` and `'{'` close on a quote of their own,
/// while the `'a` in a borrow never does. Only the literal needs blanking — it
/// is the one that can carry a brace.
fn character_literal_width(source: &[char], index: usize) -> Option<usize> {
    if source.get(index) != Some(&'\'') {
        return None;
    }
    if source.get(index + 1) == Some(&'\\') {
        let close = (index + 2..source.len()).find(|&cursor| source[cursor] == '\'')?;
        return Some(close + 1 - index);
    }
    (source.get(index + 2) == Some(&'\'')).then_some(3)
}

/// Whether a raw literal's closing quote is followed by its hashes.
fn closes_raw(source: &[char], index: usize, hashes: usize) -> bool {
    (index..index + hashes).all(|cursor| source.get(cursor) == Some(&'#'))
}

impl Default for SourceTreeScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scanner rooted at an arbitrary directory, for fixture trees.
    fn scanner_rooted_at(root: &Path) -> SourceTreeScanner {
        SourceTreeScanner {
            root: root.to_path_buf(),
            skip_dirs: Vec::new(),
        }
    }

    /// Write one fixture file, scan it, and return each line with its flag.
    fn classify(source: &str) -> Vec<(String, bool)> {
        let root = tempfile::tempdir().expect("a temporary directory for the fixture");
        std::fs::write(root.path().join("fixture.rs"), source).expect("the fixture file is writable");

        let mut seen = Vec::new();
        scanner_rooted_at(root.path()).for_each_line(|visit| seen.push((visit.line.to_owned(), visit.in_cfg_test)));
        seen
    }

    /// The flag carried by the one fixture line containing `marker`.
    fn flag_for(seen: &[(String, bool)], marker: &str) -> bool {
        let found: Vec<_> = seen.iter().filter(|(line, _)| line.contains(marker)).collect();
        assert_eq!(found.len(), 1, "the fixture names `{marker}` exactly once");
        found[0].1
    }

    /// A scan root that cannot be read is a failed audit, not an empty tree.
    ///
    /// ´claim:audit:a-source-audit-that-cannot-read-the-tree-fails-instead-of-reporting-it-clean´
    /// ´test:unit:a-missing-scan-root-fails-the-audit-naming-the-path´
    #[test]
    #[should_panic(expected = "no-such-directory-under-the-source-audit")]
    fn a_missing_scan_root_fails_the_audit_naming_the_path() {
        let missing = manifest_dir().join("no-such-directory-under-the-source-audit");
        scanner_rooted_at(&missing).for_each_line(|_| {});
    }

    /// A source file that cannot be read is a failed audit, not a skipped file.
    ///
    /// (´claim:audit:a-source-audit-that-cannot-read-the-tree-fails-instead-of-reporting-it-clean´)
    /// ´test:unit:an-unreadable-source-file-fails-the-audit-naming-the-path´
    #[test]
    #[should_panic(expected = "not-valid-utf8.rs")]
    fn an_unreadable_source_file_fails_the_audit_naming_the_path() {
        let root = tempfile::tempdir().expect("a temporary directory for the fixture");
        std::fs::write(root.path().join("not-valid-utf8.rs"), [0x66_u8, 0x6e, 0xff, 0xfe]).expect("the fixture file is writable");

        scanner_rooted_at(root.path()).for_each_line(|_| {});
    }

    /// A guarded enum variant covers the variant and stops there.
    ///
    /// ´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´
    /// ´test:unit:a-guarded-variant-does-not-hide-the-items-after-it´
    #[test]
    fn a_guarded_variant_does_not_hide_the_items_after_it() {
        let seen = classify(
            "
pub enum Command {
    Run,
    #[cfg(test)]
    ForceDecay {
        factor: f64,
    },
}

pub struct AfterTheEnum {
    field: u8,
}
",
        );

        assert!(flag_for(&seen, "ForceDecay"), "the guarded variant is test-only");
        assert!(flag_for(&seen, "factor: f64"), "and so are the fields it carries");
        assert!(!flag_for(&seen, "AfterTheEnum"), "the item after the variant is production");
        assert!(!flag_for(&seen, "field: u8"), "and so is its body");
    }

    /// A guarded declaration ends at its semicolon, not at the end of the file.
    ///
    /// (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´)
    /// ´test:unit:a-guarded-declaration-does-not-hide-the-rest-of-the-file´
    #[test]
    fn a_guarded_declaration_does_not_hide_the_rest_of_the_file() {
        let seen = classify(
            "
#[cfg(test)]
mod tests;

pub fn after_the_declaration() {}
",
        );

        assert!(flag_for(&seen, "mod tests;"), "the guarded declaration is test-only");
        assert!(
            !flag_for(&seen, "after_the_declaration"),
            "the rest of the file is production"
        );
    }

    /// A guarded module covers its block and stops at the closing brace.
    ///
    /// (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´)
    /// ´test:unit:a-guarded-module-does-not-hide-the-items-after-it´
    #[test]
    fn a_guarded_module_does_not_hide_the_items_after_it() {
        let seen = classify(
            r#"
#[cfg(test)]
mod tests {
    fn helper() {
        let braces = "{}";
    }
}

pub fn after_the_module() {}
"#,
        );

        assert!(flag_for(&seen, "mod tests {"), "the guarded module is test-only");
        assert!(flag_for(&seen, "let braces"), "and so is its body");
        assert!(
            !flag_for(&seen, "after_the_module"),
            "the item after the module is production"
        );
    }

    /// Braces nested in the guarded item, or written inside one of its
    /// literals, do not close it early.
    ///
    /// (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´)
    /// ´test:unit:nested-braces-inside-a-guarded-item-do-not-end-it-early´
    #[test]
    fn nested_braces_inside_a_guarded_item_do_not_end_it_early() {
        let seen = classify(
            r#"
#[cfg(test)]
impl Fixture {
    fn nested() {
        if true {
            let unbalanced = "}}}";
        }
    }
}

pub const AFTER: u8 = 0;
"#,
        );

        assert!(flag_for(&seen, "fn nested"), "a nested block does not close the item");
        assert!(flag_for(&seen, "let unbalanced"), "nor does a brace inside a literal");
        assert!(!flag_for(&seen, "AFTER"), "the item after the guarded block is production");
    }

    /// The audits read the production lines that follow a guarded variant.
    ///
    /// The maintenance loop guards one enum variant near the top of a long
    /// file. Under a flag that never came down, every line after that variant
    /// — the dimension state, the loop body, the whole production remainder —
    /// was classified as test code, and the architecture audits that ask what
    /// a core module may name skipped all of it. The count is asserted as a
    /// floor rather than as a figure, so that editing the maintenance loop
    /// does not fail a test about the scanner.
    ///
    /// (´claim:audit:a-cfg-test-attribute-covers-the-item-it-guards-and-the-lines-after-it-stay-production´)
    /// ´test:unit:a-guarded-variant-no-longer-hides-the-maintenance-loop´
    #[test]
    fn a_guarded_variant_no_longer_hides_the_maintenance_loop() {
        let mut first_guard = None;
        let mut production_after = 0_usize;
        let mut reaches_dimension_state = false;

        SourceTreeScanner::new().for_each_line(|visit| {
            if visit.path.file_name().and_then(|name| name.to_str()) != Some("maintenance_loop.rs") {
                return;
            }

            let trimmed = visit.line.trim_start();
            if first_guard.is_none() && trimmed.starts_with("#[cfg(test)]") {
                first_guard = Some(visit.line_no);
            }
            let Some(guard_line) = first_guard else { return };
            if visit.line_no <= guard_line || visit.in_cfg_test || trimmed.is_empty() || trimmed.starts_with("//") {
                return;
            }

            production_after += 1;
            if trimmed.starts_with("struct DimensionState") {
                reaches_dimension_state = true;
            }
        });

        assert!(
            first_guard.is_some(),
            "the maintenance loop still guards an item with `cfg(test)`"
        );
        assert!(
            production_after > 0,
            "every line after the first guarded item is being read as test code"
        );
        assert!(
            reaches_dimension_state,
            "the production state the loop is built on is read as production: {production_after} production lines follow the guard"
        );
    }
}
