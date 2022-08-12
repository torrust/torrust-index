CREATE TABLE IF NOT EXISTS torrust_torrent_pieces (
    piece_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    torrent_id INTEGER NOT NULL,
    key_in_files INTEGER NOT NULL DEFAULT 0,
    hash TEXT NOT NULL,
    is_root_hash BOOLEAN DEFAULT FALSE,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)
