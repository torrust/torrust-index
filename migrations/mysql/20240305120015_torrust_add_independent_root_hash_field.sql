ALTER TABLE torrust_torrents ADD COLUMN root_hash LONGTEXT;

-- Make `pieces` nullable. BEP 30 torrents does have this field.
ALTER TABLE torrust_torrents MODIFY COLUMN pieces LONGTEXT;
