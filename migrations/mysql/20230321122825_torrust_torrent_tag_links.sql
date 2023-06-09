CREATE TABLE IF NOT EXISTS torrust_torrent_tag_links (
    torrent_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    FOREIGN KEY (torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES torrust_torrent_tags(tag_id) ON DELETE CASCADE,
    PRIMARY KEY (torrent_id, tag_id)
);
