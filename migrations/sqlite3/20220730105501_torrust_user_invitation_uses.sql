CREATE TABLE IF NOT EXISTS torrust_user_invitations (
    invitation_use_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    invitation_id INTEGER NOT NULL,
    registered_user_id INTEGER NOT NULL,
    date_used TEXT NOT NULL,
    FOREIGN KEY(invitation_id) REFERENCES torrust_user_invitations(invitation_id),
    FOREIGN KEY(registered_user_id) REFERENCES torrust_users(user_id)
)
