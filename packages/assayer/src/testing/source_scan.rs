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
    /// `true` once a `#[cfg(test)]` attribute has been seen earlier in
    /// this file. A coarse approximation that matches existing audits.
    pub in_cfg_test: bool,
}

/// Builder for walking the crate's `src/` tree line-by-line.
///
/// Created via [`SourceTreeScanner::new`], which roots the walk at
/// `$CARGO_MANIFEST_DIR/src` as resolved by [`manifest_dir`] — the
/// invoking checkout, read at runtime. Directories and individual file
/// names can be skipped; the visitor is invoked for every remaining
/// line in every `.rs` file.
pub struct SourceTreeScanner {
    root: PathBuf,
    skip_dirs: Vec<String>,
    skip_files: Vec<String>,
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
            skip_files: Vec::new(),
        }
    }

    /// Skip any directory whose final component equals `name`.
    #[must_use]
    pub fn skip_dir(mut self, name: impl Into<String>) -> Self {
        self.skip_dirs.push(name.into());
        self
    }

    /// Skip any file whose name equals `name` (e.g. `"numerics.rs"`).
    #[must_use]
    pub fn skip_file(mut self, name: impl Into<String>) -> Self {
        self.skip_files.push(name.into());
        self
    }

    /// The directory the scan walks. Useful for assertions such as
    /// "`src/stubs.rs` stays deleted".
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Walk the tree and invoke `visit` once per line of every `.rs` file.
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
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
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
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if self.skip_files.iter().any(|s| s == filename) {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };

            let mut in_cfg_test = false;
            for (idx, line) in content.lines().enumerate() {
                if line.trim_start().starts_with("#[cfg(test)]") {
                    in_cfg_test = true;
                }
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

impl Default for SourceTreeScanner {
    fn default() -> Self {
        Self::new()
    }
}
