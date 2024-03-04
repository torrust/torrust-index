CREATE TABLE IF NOT EXISTS torrust_torrent_nodes (
    node_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    torrent_id INTEGER NOT NULL,
    node_ip TEXT NOT NULL,
    node_port INTEGER NOT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)