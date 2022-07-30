CREATE TABLE IF NOT EXISTS torrust_user_invitations (
    invitation_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    public_key TEXT NOT NULL,
    signed_digest TEXT NOT NULL,
    date_begin TEXT NOT NULL,
    date_expiry TEXT NOT NULL,
    max_uses INTEGER NOT NULL,
    personal_message TEXT,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id),
    FOREIGN KEY(public_key) REFERENCES torrust_user_public_keys(public_key)
)
