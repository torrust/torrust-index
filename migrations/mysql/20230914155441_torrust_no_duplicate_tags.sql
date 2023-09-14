-- Step 1 & 2: Identify and update the duplicate names
UPDATE torrust_torrent_tags
JOIN (
    SELECT name
    FROM torrust_torrent_tags
    GROUP BY name
    HAVING COUNT(*) > 1
) AS DuplicateNames ON torrust_torrent_tags.name = DuplicateNames.name
SET torrust_torrent_tags.name = CONCAT(torrust_torrent_tags.name, '_', torrust_torrent_tags.tag_id);

-- Step 3: Add the UNIQUE constraint to the name column
ALTER TABLE torrust_torrent_tags ADD UNIQUE (name);
