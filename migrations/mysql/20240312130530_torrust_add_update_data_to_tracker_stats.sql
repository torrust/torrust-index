-- New field to track when stats were updated from the tracker
ALTER TABLE torrust_torrent_tracker_stats ADD COLUMN updated_at DATETIME DEFAULT NULL;
UPDATE torrust_torrent_tracker_stats SET updated_at = '1000-01-01 00:00:00';
ALTER TABLE torrust_torrent_tracker_stats MODIFY COLUMN updated_at DATETIME NOT NULL;