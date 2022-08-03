CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT ,
    uploader VARCHAR(32) NOT NULL,
    info_hash CHAR(40) UNIQUE NOT NULL,
    title VARCHAR(256) UNIQUE NOT NULL,
    category_id INTEGER NOT NULL,
    description TEXT,
    upload_date BIGINT NOT NULL,
    file_size BIGINT NOT NULL,
    seeders INTEGER NOT NULL,
    leechers INTEGER NOT NULL,
    FOREIGN KEY(uploader) REFERENCES torrust_user_profiles(username) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES torrust_categories(category_id) ON DELETE CASCADE
)

