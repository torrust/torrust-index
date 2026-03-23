# ADR-T-001: Lowercase Infohashes

**Status:** Decided
**Date:** 2023-05-10
**Relates to:** [ADR-T-002](002-ignore-non-standard-info-fields.md)
(info dictionary handling — shares the infohash-correctness concern)

## Context

The index historically used both uppercase and lowercase hex
representations for BitTorrent infohashes. Because infohashes are
compared as raw byte strings in the database and in API URLs, mixed
casing creates a class of bugs: a torrent inserted with an uppercase
infohash will not be found by a lowercase query, and vice versa.
Every code path that touches an infohash must remember to normalise
case first — an easy source of defects.

Choosing a single canonical casing eliminates the problem at the
source.

## Options Considered

| Option          | Convention   | Precedent / Rationale                                           |
| --------------- | ------------ | --------------------------------------------------------------- |
| A. Uppercase    | `A1B2C3…`    | Some tracker APIs return uppercase; existing DB used uppercase. |
| B. Lowercase    | `a1b2c3…`    | SHA-1/SHA-256 hashes are conventionally printed in lowercase.   |
| C. Case-folding | either, fold | Store as-is, compare case-insensitively.                        |

Option C was rejected because it pushes complexity into every
query — `LOWER()` / `COLLATE NOCASE` — and makes URL routing
ambiguous.

## Decision

**Option B — Lowercase everywhere.**

Infohashes are normalised to lowercase hexadecimal as early as
possible after entering the system (API input, torrent upload,
tracker sync). All internal representations, database columns, and
API responses use lowercase.

### Migration

Existing uppercase infohashes were converted in-place:

```sql
-- migrations/{mysql,sqlite3}/20230627144318_torrust_covert_infohashes_to_lowercase.sql
UPDATE torrust_torrents SET info_hash = LOWER(info_hash);
```

New code paths normalise at the boundary (deserialisers, upload
handlers) so the invariant is maintained without downstream checks.

## Consequences

- Single canonical form removes an entire class of comparison bugs.
- Database indices work without case-folding functions.
- API URLs are stable and unambiguous.
- Existing clients that relied on uppercase responses need to update
  (breaking change, hence the migration).
