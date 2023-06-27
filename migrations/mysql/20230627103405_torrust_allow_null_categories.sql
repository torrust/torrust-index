-- Step 1: Allow null categories for torrents
ALTER TABLE torrust_torrents MODIFY category_id INTEGER NULL;

-- Step 2: Set torrent category to NULL when category is deleted
ALTER TABLE `torrust_torrents` DROP FOREIGN KEY `torrust_torrents_ibfk_2`;
ALTER TABLE `torrust_torrents` ADD CONSTRAINT `torrust_torrents_ibfk_2` FOREIGN KEY (`category_id`) REFERENCES `torrust_categories` (`category_id`) ON DELETE SET NULL;
