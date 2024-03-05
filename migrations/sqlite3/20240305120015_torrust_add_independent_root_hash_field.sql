-- add field `root_hash` and make `pieces` nullable
CREATE TABLE
    "torrust_torrents_new" (
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

-- Step 2: Copy data from the old table to the new table
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

-- Step 3: Drop the old table
DROP TABLE torrust_torrents;

-- Step 4: Rename the new table to the original name
ALTER TABLE torrust_torrents_new
RENAME TO torrust_torrents;