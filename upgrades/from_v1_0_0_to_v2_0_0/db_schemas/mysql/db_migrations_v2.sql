# 20220721205537_torrust_users.sql

CREATE TABLE IF NOT EXISTS torrust_users (
    user_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    date_registered DATETIME NOT NULL,
    administrator BOOLEAN NOT NULL DEFAULT FALSE
)

# 20220721210530_torrust_user_authentication.sql

CREATE TABLE IF NOT EXISTS torrust_user_authentication (
    user_id INTEGER NOT NULL PRIMARY KEY,
    password_hash TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)

# 20220727213942_torrust_user_profiles.sql

CREATE TABLE IF NOT EXISTS torrust_user_profiles (
    user_id INTEGER NOT NULL PRIMARY KEY,
    username VARCHAR(24) NOT NULL UNIQUE,
    email VARCHAR(320) UNIQUE,
    email_verified BOOL NOT NULL DEFAULT FALSE,
    bio TEXT,
    avatar TEXT,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)

# 20220727222313_torrust_tracker_keys.sql

CREATE TABLE IF NOT EXISTS torrust_tracker_keys (
    tracker_key_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    tracker_key CHAR(32) NOT NULL,
    date_expiry BIGINT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)

# 20220730102607_torrust_user_public_keys.sql

CREATE TABLE IF NOT EXISTS torrust_user_public_keys (
    public_key_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    public_key CHAR(32) UNIQUE NOT NULL,
    date_registered DATETIME NOT NULL,
    date_expiry DATETIME NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)

# 20220730104552_torrust_user_invitations.sql

CREATE TABLE IF NOT EXISTS torrust_user_invitations (
    invitation_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    public_key CHAR(32) NOT NULL,
    signed_digest CHAR(32) NOT NULL,
    date_begin DATETIME NOT NULL,
    date_expiry DATETIME NOT NULL,
    max_uses INTEGER NOT NULL,
    personal_message VARCHAR(512),
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(public_key) REFERENCES torrust_user_public_keys(public_key) ON DELETE CASCADE
)

# 20220730105501_torrust_user_invitation_uses.sql

CREATE TABLE IF NOT EXISTS torrust_user_invitation_uses (
    invitation_use_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    invitation_id INTEGER NOT NULL,
    registered_user_id INTEGER NOT NULL,
    date_used DATETIME NOT NULL,
    FOREIGN KEY(invitation_id) REFERENCES torrust_user_invitations(invitation_id) ON DELETE CASCADE,
    FOREIGN KEY(registered_user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)

# 20220801201435_torrust_user_bans.sql

CREATE TABLE IF NOT EXISTS torrust_user_bans (
    ban_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    reason TEXT NOT NULL,
    date_expiry DATETIME NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)

# 20220802161524_torrust_categories.sql

CREATE TABLE torrust_categories (
    category_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(64) NOT NULL UNIQUE
);

INSERT INTO torrust_categories (name) VALUES ('movies'), ('tv shows'), ('games'), ('music'), ('software');

# 20220810192613_torrust_torrents.sql

CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    uploader_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    info_hash CHAR(40) UNIQUE NOT NULL,
    size BIGINT NOT NULL,
    name TEXT NOT NULL,
    pieces LONGTEXT NOT NULL,
    piece_length BIGINT NOT NULL,
    private BOOLEAN NULL DEFAULT NULL,
    root_hash BOOLEAN NOT NULL DEFAULT FALSE,
    date_uploaded DATETIME NOT NULL,
    FOREIGN KEY(uploader_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES torrust_categories(category_id) ON DELETE CASCADE
)

# 20220810201538_torrust_torrent_files.sql

CREATE TABLE IF NOT EXISTS torrust_torrent_files (
    file_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    torrent_id INTEGER NOT NULL,
    md5sum TEXT NULL DEFAULT NULL,
    length BIGINT NOT NULL,
    path TEXT DEFAULT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)

# 20220810201609_torrust_torrent_announce_urls.sql

CREATE TABLE IF NOT EXISTS torrust_torrent_announce_urls (
    announce_url_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    torrent_id INTEGER NOT NULL,
    tracker_url VARCHAR(256) NOT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)

# 20220812181520_torrust_torrent_info.sql

CREATE TABLE IF NOT EXISTS torrust_torrent_info (
    torrent_id INTEGER NOT NULL PRIMARY KEY,
    title VARCHAR(256) UNIQUE NOT NULL,
    description TEXT DEFAULT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)

# 20220812184806_torrust_torrent_tracker_stats.sql

CREATE TABLE IF NOT EXISTS torrust_torrent_tracker_stats (
    torrent_id INTEGER NOT NULL PRIMARY KEY,
    tracker_url VARCHAR(256) NOT NULL,
    seeders INTEGER NOT NULL DEFAULT 0,
    leechers INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE,
    UNIQUE(torrent_id, tracker_url)
)
