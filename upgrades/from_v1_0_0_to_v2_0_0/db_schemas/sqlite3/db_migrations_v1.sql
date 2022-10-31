# 20210831113004_torrust_users.sql

CREATE TABLE IF NOT EXISTS torrust_users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(32) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    password TEXT NOT NULL
);

# 20210904135524_torrust_tracker_keys.sql

CREATE TABLE IF NOT EXISTS torrust_tracker_keys (
    key_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    key VARCHAR(32) NOT NULL,
    valid_until INT(10) NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id)
);

# 20210905160623_torrust_categories.sql

CREATE TABLE torrust_categories (
    category_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(64) NOT NULL UNIQUE
);

INSERT INTO torrust_categories (name) VALUES
('movies'), ('tv shows'), ('games'), ('music'), ('software');

# 20210907083424_torrust_torrent_files.sql

CREATE TABLE IF NOT EXISTS torrust_torrent_files (
    file_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    torrent_id INTEGER NOT NULL,
    number INTEGER NOT NULL,
    path VARCHAR(255) NOT NULL,
    length INTEGER NOT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id)
);

# 20211208143338_torrust_users.sql

ALTER TABLE torrust_users;
ADD COLUMN administrator BOOLEAN NOT NULL DEFAULT FALSE;

# 20220308083424_torrust_torrents.sql

CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uploader VARCHAR(32) NOT NULL,
    info_hash VARCHAR(20) UNIQUE NOT NULL,
    title VARCHAR(256) UNIQUE NOT NULL,
    category_id INTEGER NOT NULL,
    description TEXT,
    upload_date INT(10) NOT NULL,
    file_size BIGINT NOT NULL,
    seeders INTEGER NOT NULL,
    leechers INTEGER NOT NULL,
    FOREIGN KEY(uploader) REFERENCES torrust_users(username) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES torrust_categories(category_id) ON DELETE CASCADE
);

# 20220308170028_torrust_categories.sql

ALTER TABLE torrust_categories
ADD COLUMN icon VARCHAR(32);

