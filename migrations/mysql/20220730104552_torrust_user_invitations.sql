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
