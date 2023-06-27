-- Step 1: Create a new table with the new structure
CREATE TABLE IF NOT EXISTS "torrust_torrents_new" (
	"torrent_id" INTEGER NOT NULL,
	"uploader_id" INTEGER NOT NULL,
	"category_id" INTEGER NULL,
	"info_hash" TEXT NOT NULL UNIQUE,
	"size" INTEGER NOT NULL,
	"name" TEXT NOT NULL,
	"pieces" TEXT NOT NULL,
	"piece_length" INTEGER NOT NULL,
	"private" BOOLEAN DEFAULT NULL,
	"root_hash" INT NOT NULL DEFAULT 0,
	"date_uploaded" TEXT NOT NULL,
	FOREIGN KEY("uploader_id") REFERENCES "torrust_users"("user_id") ON DELETE CASCADE,
	FOREIGN KEY("category_id") REFERENCES "torrust_categories"("category_id") ON DELETE SET NULL,
	PRIMARY KEY("torrent_id" AUTOINCREMENT)
);

-- Step 2: Copy rows from the current table to the new table
INSERT INTO torrust_torrents_new (torrent_id, uploader_id, category_id, info_hash, size, name, pieces, piece_length, private, root_hash, date_uploaded)
SELECT torrent_id, uploader_id, category_id, info_hash, size, name, pieces, piece_length, private, root_hash, date_uploaded
FROM torrust_torrents;

-- Step 3: Delete the current table
DROP TABLE torrust_torrents;

-- Step 1: Rename the new table
ALTER TABLE torrust_torrents_new RENAME TO torrust_torrents;
