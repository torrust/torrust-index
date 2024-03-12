-- New field to track when stats were updated from the tracker
ALTER TABLE torrust_torrent_tracker_stats ADD COLUMN updated_at TEXT DEFAULT "1000-01-01 00:00:00";
