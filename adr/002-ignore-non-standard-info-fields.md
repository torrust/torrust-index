# ADR-T-002: Ignore Non-Standard Fields in Info Dictionary

**Status:** Decided (temporary — revisit when full-fidelity storage
is feasible)
**Date:** 2023-08-24
**Relates to:** [ADR-T-001](001-lowercase-infohashes.md) (infohash
canonicalisation — non-standard fields affect the computed infohash)

## Context

BitTorrent infohashes are the SHA-1 (v1) or SHA-256 (v2) digest of
the bencoded `info` dictionary. The digest covers _every_ key in the
dictionary, including non-standard ones. Some real-world torrents
carry custom keys such as `collections` (Internet Archive), `x_cross_seed`,
or tracker-specific metadata.

The index deserialises the `info` dictionary into a fixed Rust struct
(`TorrentInfoDictionary` in
[src/models/torrent_file.rs](../../src/models/torrent_file.rs)):

```rust
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfoDictionary {
    pub name: String,
    pub pieces: Option<ByteBuf>,
    pub piece_length: i64,
    pub md5sum: Option<String>,
    pub length: Option<i64>,
    pub files: Option<Vec<TorrentFile>>,
    pub private: Option<u8>,
    pub path: Option<Vec<String>>,
    pub root_hash: Option<String>,
    pub source: Option<String>,
}
```

Any key not listed above is silently dropped by `serde`. When the
index later re-encodes the struct to compute the infohash, the
missing keys change the bencoded output and produce a **different
infohash** from the original torrent. This is a data-integrity
problem: the indexed torrent's identity no longer matches the
torrent circulating on the swarm.

## Options Considered

| Option                           | Strategy                                                                        | Infohash fidelity                | Complexity                                                     |
| -------------------------------- | ------------------------------------------------------------------------------- | -------------------------------- | -------------------------------------------------------------- |
| A. Ignore non-standard fields    | Deserialise into fixed struct, drop unknowns                                    | Broken for affected torrents     | Low                                                            |
| B. Preserve raw info bytes       | Store the original bencoded `info` blob; use it for infohash computation        | Correct                          | Medium — requires raw-byte capture alongside structured fields |
| C. `serde` `flatten` + `HashMap` | Capture unknown keys via `#[serde(flatten)]` into a side map; re-emit on encode | Mostly correct (ordering issues) | Medium — bencode canonical ordering must be enforced           |
| D. Full bencode round-trip       | Use a bencode library that preserves unknown keys natively                      | Correct                          | High — may require replacing the current bencode stack         |

## Decision

**Option A — Ignore non-standard fields (temporary).**

This is a pragmatic, short-term choice. The affected torrent
population is small, and the alternative options require significant
refactoring of the torrent ingest pipeline. The decision is
explicitly marked temporary; Option B or D should replace it when
full-fidelity storage is implemented.

### Mitigations

1. **User-facing warning** — the upload flow warns that non-standard
   info fields will be stripped.
2. **Documentation** — this ADR and the API docs note the limitation.

### Source field exception

The `source` key is technically non-standard (not defined in any
BEP), but it is widely used by private trackers to differentiate
swarms. It is therefore included in `TorrentInfoDictionary` and
preserved during round-trip.

## Consequences

- Torrents with non-standard info fields will have a different
  infohash after indexing — a known data-integrity gap.
- The struct-based approach remains simple and type-safe.
- A future ADR should revisit this decision when raw-info-blob
  storage (Option B) or a round-trip-safe bencode library (Option D)
  is adopted.
