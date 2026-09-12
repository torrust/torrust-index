#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# SPDX-FileCopyrightText: 2026 Torrust project contributors
#
# Feed-forward CI lint for the Assayer crate.
#
# Enforces two invariants:
#   1. torrust_sentinel enters the Assayer only by naming permitted flat
#      report types directly — `use torrust_sentinel::Name;` or
#      `use torrust_sentinel::{Names};` over the whitelist — and is otherwise
#      referred to only by fully-qualified paths to those same names. Every
#      other import form is a violation: an alias at any depth, a glob, a bare
#      import of the crate, an `extern crate`, a re-export, and a path with a
#      module segment in it. Each of those puts a name into scope that carries
#      no torrust_sentinel token where it is used, so a path check alone
#      cannot see it and the boundary has to be held at the import.
#      GNodeId is imported from torrust_mudlark directly.
#   2. No direct .powf() calls outside numerics.rs and bridge.rs.
#
# Usage:
#   ./packages/assayer/ci/lint_assayer.sh
#
# Exit code 0 = clean, 1 = violations found.

set -euo pipefail

ASSAYER_SRC="packages/assayer/src"
EXIT_CODE=0

# ─── Lint 1: Feed-forward boundary enforcement ───────────────────────────────
# Only flat report/readout re-exports from torrust_sentinel are permitted:
#   use torrust_sentinel::{BatchReport, CellReport, ...}
# Fully-qualified references to those flat re-exports are also allowed.
# GNodeId is imported from torrust_mudlark directly.
#
# Every import statement is accumulated to its terminator and then judged as a
# whole, because a form's evasion is a property of the statement rather than of
# any one of its lines. The path check that follows only sees text spelling
# torrust_sentinel, so anything that binds the crate or one of its modules to
# another name is judged here or not at all.

echo "=== Lint: torrust_sentinel feed-forward boundary ==="

SENTINEL_VIOLATIONS=$(find "$ASSAYER_SRC" -type f -name '*.rs' -print0 \
    | xargs -0 perl -0ne '
        BEGIN {
            %allowed = map { $_ => 1 } qw(
                AnalysisSetSummary AnomalyScores AxisBaselineSnapshots BaselineSnapshot
                BatchReport CellInspection CellReport ClipPressureDistribution
                ContourSnapshot CoordinationHealth CoordinationReport CusumSnapshot
                GeometryDistribution HealthReport MaturityDistribution MemberScore
                RankDistribution SampleScore ScoreDistribution ScoringGeometry
                TrackerMaturity TrackerReport
            );
        }

        my @lines = split /(?<=\n)/, $_;
        my $in_use = 0;
        my $use_statement = "";
        my $use_start_line = 0;

        for (my $i = 0; $i < @lines; $i++) {
            my $line = $lines[$i];
            my $source = $line;
            $source =~ s{//.*}{};

            if ($source =~ /\b(?:SpectralSentinel|SentinelConfig|SubspaceTracker)\b/) {
                print "$ARGV:" . ($i + 1) . ":$line";
                # A statement being accumulated is reported by this line
                # already; its terminator still closes it, so the lines after
                # it are read as code rather than swallowed into an import
                # that never ends.
                if ($in_use && $source =~ /;/) {
                    $in_use = 0;
                    $use_statement = "";
                }
                next;
            }

            if ($in_use) {
                $use_statement .= $source;
                if ($source =~ /;/) {
                    check_sentinel_import($use_statement, $use_start_line, $lines[$use_start_line - 1]);
                    $in_use = 0;
                    $use_statement = "";
                }
                next;
            }

            # Every import is taken, not only the ones already spelling the
            # crate: an alias or a glob is exactly the statement whose own
            # text stops naming it.
            if ($source =~ /^\s*(?:pub\s*(?:\([^)]*\)\s*)?)?(?:use|extern\s+crate)\s/) {
                $use_statement = $source;
                $use_start_line = $i + 1;
                if ($source =~ /;/) {
                    check_sentinel_import($use_statement, $use_start_line, $line);
                    $use_statement = "";
                } else {
                    $in_use = 1;
                }
                next;
            }

            check_sentinel_paths($source, $i + 1, $line);
        }

        sub check_sentinel_import {
            my ($statement, $line_no, $original_line) = @_;

            $statement =~ s/\s+/ /g;
            return unless $statement =~ /(?<![A-Za-z0-9_])torrust_sentinel(?![A-Za-z0-9_])/;

            # An extern crate binds the whole crate under some name, which is
            # never an import of permitted items whatever the name is.
            if ($statement =~ /^ ?(?:pub ?(?:\([^)]*\) ?)?)?extern crate\b/) {
                print "$ARGV:$line_no:$original_line";
                return;
            }

            # A re-export puts the crate surface back onto the surface of this
            # one, so the boundary it crosses is the one the whitelist holds.
            if ($statement =~ /^ ?pub\b/) {
                print "$ARGV:$line_no:$original_line";
                return;
            }

            # The permitted shape names the crate once and then one item or a
            # braced list of them. Anything else — a bare import of the crate,
            # an alias of it, a module segment, a nested group — fails here.
            if ($statement !~ /^ ?use (?:::)?torrust_sentinel::(.+);\s*$/) {
                print "$ARGV:$line_no:$original_line";
                return;
            }

            my $body = $1;
            $body =~ s/^ //;
            $body =~ s/ $//;
            if ($body =~ /^\{(.*)\}$/) {
                $body = $1;
            }

            # Each item must be a bare permitted name. An alias keeps its "as"
            # in the item, a glob its star, a nested path its separator and a
            # nested group its brace, so each of them fails this one test.
            foreach my $item (split /,/, $body) {
                $item =~ s/^ //;
                $item =~ s/ $//;
                next if $item eq "";
                unless ($item =~ /^[A-Za-z_][A-Za-z0-9_]*$/ && $allowed{$item}) {
                    print "$ARGV:$line_no:$original_line";
                    return;
                }
            }
        }

        sub check_sentinel_paths {
            my ($source, $line_no, $original_line) = @_;

            while ($source =~ /(?<![A-Za-z0-9_])(?:::)?torrust_sentinel::([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)?)/g) {
                my $path = $1;
                my @segments = split /::/, $path;
                if (@segments != 1 || !$allowed{$segments[0]}) {
                    print "$ARGV:$line_no:$original_line";
                    return;
                }
            }
        }
    ' \
    || true)

if [ -n "$SENTINEL_VIOLATIONS" ]; then
    echo "FAIL: Prohibited torrust_sentinel boundary references found:"
    echo "$SENTINEL_VIOLATIONS"
    EXIT_CODE=1
else
    echo "PASS: All torrust_sentinel references are within the whitelist."
fi

echo ""

# ─── Lint 2: Centralised decay computation ───────────────────────────────────
# All decay computations must use the centralised functions in numerics.rs.
# Direct .powf() calls are only permitted in numerics.rs and bridge.rs.

echo "=== Lint: centralised .powf() usage ==="

POWF_VIOLATIONS=$(grep -rn '\.powf(' "$ASSAYER_SRC/" \
    | grep -v 'numerics.rs' \
    | grep -v 'bridge.rs' \
    | grep -v '/tests/' \
    | grep -v '^\s*//' \
    || true)

if [ -n "$POWF_VIOLATIONS" ]; then
    echo "FAIL: Direct .powf() usage found outside numerics.rs/bridge.rs:"
    echo "$POWF_VIOLATIONS"
    EXIT_CODE=1
else
    echo "PASS: All .powf() calls are in numerics.rs or bridge.rs."
fi

echo ""

# ─── Summary ─────────────────────────────────────────────────────────────────

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "All Assayer CI lints passed."
else
    echo "Assayer CI lints FAILED. See above for details."
fi

exit "$EXIT_CODE"
