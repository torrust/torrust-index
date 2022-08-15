CREATE TABLE IF NOT EXISTS torrust_torrent_files (
    file_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    torrent_id INTEGER NOT NULL,
    md5sum TEXT NULL DEFAULT NULL,
    length BIGINT NOT NULL,
    path TEXT DEFAULT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)
