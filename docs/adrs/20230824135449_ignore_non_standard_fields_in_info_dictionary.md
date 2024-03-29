# Ignore non-standard fields in info dictionary

This is a temporary solution to avoid problems with non-standard fields in the
info dictionary. In the future, we could add support for them.

## Context

In torrents, custom fields in the info dictionary can lead to mismatches in our system.

## Problem

Some torrents might include custom fields in the info dictionary. **Parsing non-standard fields generates a different info-hash for the indexed torrent**, leading to potential issues and misrepresentations.

A sample JSON version of a torrent with a `collections` custom field int the `info` dictionary:

```json
{
   "announce": "https://academictorrents.com/announce.php",
   "announce-list": [
      [
         "https://academictorrents.com/announce.php"
      ],
      [
         "https://ipv6.academictorrents.com/announce.php"
      ],
      [
         "udp://tracker.opentrackr.org:1337/announce"
      ],
      [
         "udp://tracker.openbittorrent.com:80/announce"
      ],
      [
         "http://bt1.archive.org:6969/announce"
      ],
      [
         "http://bt2.archive.org:6969/announce"
      ]
   ],
   "comment": "This content hosted at the Internet Archive at https://archive.org/details/rapppid-weights.tar\nFiles may have changed, which prevents torrents from downloading correctly or completely; please check for an updated torrent at https://archive.org/download/rapppid-weights.tar/rapppid-weights.tar_archive.torrent\nNote: retrieval usually requires a client that supports webseeding (GetRight style).\nNote: many Internet Archive torrents contain a 'pad file' directory. This directory and the files within it may be erased once retrieval completes.\nNote: the file rapppid-weights.tar_meta.xml contains metadata about this torrent's contents.",
   "created by": "ia_make_torrent",
   "creation date": 1689273787,
   "info": {
      "collections": [
         "org.archive.rapppid-weights.tar"
      ],
      "files": [
         {
            "crc32": "57d33fcc",
            "length": 11528324,
            "md5": "e91bb4ba82695161be68f8b33ae76142",
            "mtime": "1689273730",
            "path": [
               "RAPPPID Weights.tar.gz"
            ],
            "sha1": "45970ef33cb3049a7a8629e40c8f5e5268d1dc53"
         },
         {
            "crc32": "c658fd4f",
            "length": 20480,
            "md5": "a782b2a53ba49f0d45f3dd6e35e0d593",
            "mtime": "1689273783",
            "path": [
               "rapppid-weights.tar_meta.sqlite"
            ],
            "sha1": "bcb06b3164f1d2aba22ef6046eb80f65264e9fba"
         },
         {
            "crc32": "8140a5c7",
            "length": 1044,
            "md5": "1bab21e50e06ab42d3a77d872bf252e5",
            "mtime": "1689273763",
            "path": [
               "rapppid-weights.tar_meta.xml"
            ],
            "sha1": "b2f0f2bbec34aa9140fb9ac3fcb190588a496aa3"
         }
      ],
      "name": "rapppid-weights.tar",
      "piece length": 524288,
      "pieces": "<hex>AB EC 55 6E 0F 7B E7 D3 30 0C F6 68 8C 90 6D 99 0C 3E 32 B5 2C F2 B6 7C 0C 32 52 BC 72 6F 07 1E 73 AB 76 F1 BC 32 2B FC 21 D4 7F 1A E9 72 35 40 7E C3 B4 89 09 2B ED 4B D8 B0 6C 65 8C 27 58 AE FB 72 75 73 44 37 88 28 20 D2 94 BD A4 6A F8 D2 A6 FD 02 65 1C 1C DF B8 56 6D 3A D2 7E A7 3D CA E2 49 F7 36 8D 17 77 6E 32 AD EF A5 44 C2 8F B6 9C 24 56 AD E8 FB 7B A6 71 C0 81 E5 43 03 91 D4 4F B0 A6 64 CA 29 1B 0D 1D 40 7D 39 4E 76 96 EB 01 18 F3 F5 50 8E 2F FA 54 FC 49 66 85 D8 38 87 78 9B 0A 8F 7A A3 2C 8F 72 36 AD 6D 74 0B FC C5 57 71 86 FB F3 CF CA C9 DA EC 61 62 A2 2A 1B A7 85 89 91 8F AA C0 C0 CB 0D 57 D8 B7 E7 64 4D F2 84 73 76 98 FB 3A 17 48 D7 9C 01 FE CA 6D 1F C5 97 34 05 54 39 DA C2 6E 17 41 11 69 F3 46 D1 7D 16 D3 C0 87 3B C3 B2 0C 1D E0 E2 49 C3 05 D2 4C 00 5A 5B 78 01 12 3E BF 52 43 49 6D 1A EE 23 79 D2 0E 28 B6 84 7E C5 ED 79 DE 64 02 ED 47 71 3D 93 16 C4 A2 76 18 77 54 C5 31 48 96 3A 51 C1 4A 92 90 91 F3 CF 48 5B 24 86 55 A8 EB 0C C6 2D 86 E2 29 56 09 2C 38 0B CD C1 CA 45 E6 64 6A 47 FE BB 2E 47 9A 77 45 29 E9 72 19 20 6F 42 79 2B 37 B9 53 25 ED 0F 29 04 D5 E2 96 F1 DE CF 99 BE 32 AA B8 0A 1D 0B 9F B9 D6 AB 5C 50 43 78 85 41 09 01 24 CF E0 89 76 5B 4D A9 CA 72 C0 DF 92 47 0F 0D CE CA 96 C6 7E A5 41 5F 2B A7 BB 04 CC F7 44 7F 94 1E 24 D2 1B 17 CA 18 79 90 A3 D6 20 75 A2 96 68 06 58 5A DE F5 2C 1A 90 22 72 33 8E D5 B2 A8 FA E5 E9 E7 69 62 02 7C 09 B3 4C</hex>"
   },
   "locale": "en",
   "title": "rapppid-weights.tar",
   "url-list": [
      "https://archive.org/download/",
      "http://ia902702.us.archive.org/22/items/",
      "http://ia802702.us.archive.org/22/items/"
   ]
}
```

> NOTICE: The `collections` field.

At the moment we are only parsing these fields from the `info` dictionary:

```rust
pub struct TorrentInfo {
    pub name: String,
    #[serde(default)]
    pub pieces: Option<ByteBuf>,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<TorrentFile>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}
```

> WARNING!: If the uploaded torrent has a non-standard field in the info dictionary,
> it will not only be ignore but it will produce a different info-hash for the indexed torrent.

## Agreement

1. Temporary Solution: Ignore all non-standard fields in the info dictionary.
2. Communication: Users will be alerted about this decision through UI warnings and documentation.
3. Future Consideration: There is a potential to support these fields in future iterations.

## Rationale

- Prioritizing standard fields ensures uniformity in the representation of torrents.
- Warnings and documentation provide transparency to users.
- A future-proof approach leaves room for possible expansion or reconsideration.

## Other considerations

The source field migth be considered a non-standard field, because it's not included in any BEP, but this field is being parsed and stored in the database since it seems to be widely used by private trackers.
