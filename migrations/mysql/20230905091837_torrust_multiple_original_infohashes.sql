-- Step 1: Create a new table with all infohashes
CREATE TABLE torrust_torrent_info_hashes (
    info_hash CHAR(40) NOT NULL,
    canonical_info_hash CHAR(40) NOT NULL,
    original_is_known BOOLEAN NOT NULL,
    PRIMARY KEY(info_hash),
    FOREIGN KEY(canonical_info_hash) REFERENCES torrust_torrents(info_hash) ON DELETE CASCADE
);

-- Step 2: Create one record for each torrent with only the canonical infohash.
--         The original infohash is NULL so we do not know if it was the same.
--         This happens if the uploaded torrent was uploaded before introducing
--         the feature to store the original infohash
INSERT INTO torrust_torrent_info_hashes (info_hash, canonical_info_hash, original_is_known)
SELECT info_hash, info_hash, FALSE
    FROM torrust_torrents
    WHERE original_info_hash IS NULL;

-- Step 3: Create one record for each torrent with the same original and 
--         canonical infohashes.
INSERT INTO torrust_torrent_info_hashes (info_hash, canonical_info_hash, original_is_known)
SELECT info_hash, info_hash, TRUE
    FROM torrust_torrents
    WHERE original_info_hash IS NOT NULL
        AND info_hash = original_info_hash;

-- Step 4: Create two records for each torrent with a different original and 
--         canonical infohashes. One record with the same original and canonical
--         infohashes and one record with the original infohash and the canonical
--         one.
-- Insert the canonical infohash
INSERT INTO torrust_torrent_info_hashes (info_hash, canonical_info_hash, original_is_known)
SELECT info_hash, info_hash, TRUE
    FROM torrust_torrents
    WHERE original_info_hash IS NOT NULL
        AND info_hash != original_info_hash;
-- Insert the original infohash pointing to the canonical
INSERT INTO torrust_torrent_info_hashes (info_hash, canonical_info_hash, original_is_known)
SELECT original_info_hash, info_hash, TRUE
    FROM torrust_torrents
    WHERE original_info_hash IS NOT NULL
        AND info_hash != original_info_hash;

-- Step 5: Delete the `torrust_torrents::original_info_hash` column
ALTER TABLE torrust_torrents DROP COLUMN original_info_hash;

