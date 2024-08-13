PRAGMA foreign_keys = off;

-- Step 1: backup secondary tables. They will be truncated because of the DELETE ON CASCADE
CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_files_backup" (
	"file_id"	INTEGER NOT NULL,
	"torrent_id"	INTEGER NOT NULL,
	"md5sum"	TEXT DEFAULT NULL,
	"length"	BIGINT NOT NULL,
	"path"	TEXT DEFAULT NULL
);
INSERT INTO torrust_torrent_files_backup SELECT * FROM torrust_torrent_files;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_announce_urls_backup" (
	"announce_url_id"	INTEGER NOT NULL,
	"torrent_id"	INTEGER NOT NULL,
	"tracker_url"	TEXT NOT NULL
);
INSERT INTO torrust_torrent_announce_urls_backup SELECT * FROM torrust_torrent_announce_urls;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_info_backup" (
	"torrent_id"	INTEGER NOT NULL,
	"title"	VARCHAR(256) NOT NULL UNIQUE,
	"description"	TEXT DEFAULT NULL
);
INSERT INTO torrust_torrent_info_backup SELECT * FROM torrust_torrent_info;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_tracker_stats_backup" (
	"torrent_id"	INTEGER NOT NULL,
	"tracker_url"	VARCHAR(256) NOT NULL,
	"seeders"	INTEGER NOT NULL DEFAULT 0,
	"leechers"	INTEGER NOT NULL DEFAULT 0
);
INSERT INTO torrust_torrent_tracker_stats_backup SELECT * FROM torrust_torrent_tracker_stats;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_tag_links_backup" (
	"torrent_id"	INTEGER NOT NULL,
	"tag_id"	INTEGER NOT NULL
);
INSERT INTO torrust_torrent_tag_links_backup SELECT * FROM torrust_torrent_tag_links;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_info_hashes_backup" (
	"info_hash"	TEXT NOT NULL,
	"canonical_info_hash"	TEXT NOT NULL,
	"original_is_known"	BOOLEAN NOT NULL
);
INSERT INTO torrust_torrent_info_hashes_backup SELECT * FROM torrust_torrent_info_hashes;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_http_seeds_backup" (
	"http_seed_id"	INTEGER NOT NULL,
	"torrent_id"	INTEGER NOT NULL,
	"seed_url"	TEXT NOT NULL
);
INSERT INTO torrust_torrent_http_seeds_backup SELECT * FROM torrust_torrent_http_seeds;

CREATE TEMPORARY TABLE IF NOT EXISTS "torrust_torrent_nodes_backup" (
	"node_id"	INTEGER NOT NULL,
	"torrent_id"	INTEGER NOT NULL,
	"node_ip"	TEXT NOT NULL,
	"node_port"	INTEGER NOT NULL
);
INSERT INTO torrust_torrent_nodes_backup SELECT * FROM torrust_torrent_nodes;

-- Step 2: Add field `root_hash` and make `pieces` nullable
CREATE TABLE
    IF NOT EXISTS "torrust_torrents_new" (
        "torrent_id" INTEGER NOT NULL,
        "uploader_id" INTEGER NOT NULL,
        "category_id" INTEGER,
        "info_hash" TEXT NOT NULL UNIQUE,
        "size" INTEGER NOT NULL,
        "name" TEXT NOT NULL,
        "pieces" TEXT,
        "root_hash" TEXT,
        "piece_length" INTEGER NOT NULL,
        "private" BOOLEAN DEFAULT NULL,
        "is_bep_30" INT NOT NULL DEFAULT 0,
        "date_uploaded" TEXT NOT NULL,
        "source" TEXT DEFAULT NULL,
        "comment" TEXT,
        "creation_date" BIGINT,
        "created_by" TEXT,
        "encoding" TEXT,
        FOREIGN KEY ("uploader_id") REFERENCES "torrust_users" ("user_id") ON DELETE CASCADE,
        FOREIGN KEY ("category_id") REFERENCES "torrust_categories" ("category_id") ON DELETE SET NULL,
        PRIMARY KEY ("torrent_id" AUTOINCREMENT)
    );

-- Step 3: Copy data from the old table to the new table
INSERT INTO
    torrust_torrents_new (
        torrent_id,
        uploader_id,
        category_id,
        info_hash,
        size,
        name,
        pieces,
        piece_length,
        private,
        root_hash,
        date_uploaded,
        source,
        comment,
        creation_date,
        created_by,
        encoding
    )
SELECT
    torrent_id,
    uploader_id,
    category_id,
    info_hash,
    size,
    name,
    CASE
        WHEN is_bep_30 = 0 THEN pieces
        ELSE NULL
    END,
    piece_length,
    private,
    CASE
        WHEN is_bep_30 = 1 THEN pieces
        ELSE NULL
    END,
    date_uploaded,
    source,
    comment,
    creation_date,
    created_by,
    encoding
FROM
    torrust_torrents;

-- Step 4: Drop the old table
DROP TABLE torrust_torrents;

-- Step 5: Rename the new table to the original name
ALTER TABLE torrust_torrents_new RENAME TO torrust_torrents;

-- Step 6: Repopulate secondary tables from backup tables
INSERT INTO torrust_torrent_files SELECT * FROM torrust_torrent_files_backup;
INSERT INTO torrust_torrent_announce_urls SELECT * FROM torrust_torrent_announce_urls_backup;
INSERT INTO torrust_torrent_info SELECT * FROM torrust_torrent_info_backup;
INSERT INTO torrust_torrent_tracker_stats SELECT * FROM torrust_torrent_tracker_stats_backup;
INSERT INTO torrust_torrent_tag_links SELECT * FROM torrust_torrent_tag_links_backup;
INSERT INTO torrust_torrent_info_hashes SELECT * FROM torrust_torrent_info_hashes_backup;
INSERT INTO torrust_torrent_http_seeds SELECT * FROM torrust_torrent_http_seeds_backup;
INSERT INTO torrust_torrent_nodes SELECT * FROM torrust_torrent_nodes_backup;

-- Step 7: Drop temporary secondary table backups
DROP TABLE torrust_torrent_files_backup;
DROP TABLE torrust_torrent_announce_urls_backup;
DROP TABLE torrust_torrent_info_backup;
DROP TABLE torrust_torrent_tracker_stats_backup;
DROP TABLE torrust_torrent_tag_links_backup;
DROP TABLE torrust_torrent_info_hashes_backup;
DROP TABLE torrust_torrent_http_seeds_backup;
DROP TABLE torrust_torrent_nodes_backup;

PRAGMA foreign_keys = on;