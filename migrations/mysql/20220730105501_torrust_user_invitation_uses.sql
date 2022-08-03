CREATE TABLE IF NOT EXISTS torrust_user_invitations (
    invitation_use_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    invitation_id INTEGER NOT NULL,
    registered_user_id INTEGER NOT NULL,
    date_used DATETIME NOT NULL,
    FOREIGN KEY(invitation_id) REFERENCES torrust_user_invitations(invitation_id) ON DELETE CASCADE,
    FOREIGN KEY(registered_user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)
