-- Step 1: Identify and update the duplicate names
WITH DuplicateNames AS (
    SELECT name
    FROM torrust_torrent_tags
    GROUP BY name
    HAVING COUNT(*) > 1
)
UPDATE torrust_torrent_tags
SET name = name || '_' || tag_id
WHERE name IN (SELECT name FROM DuplicateNames);

-- Step 2: Create a UNIQUE index on the name column
CREATE UNIQUE INDEX idx_unique_name ON torrust_torrent_tags(name);
