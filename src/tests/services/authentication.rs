use crate::models::user::UserAuthentication;
use crate::services::authentication::verify_password;

#[test]
fn password_hashed_with_pbkdf2_sha256_should_be_verified() {
    let password = b"12345678";
    let password_hash =
        "$pbkdf2-sha256$i=10000,l=32$pZIh8nilm+cg6fk5Ubf2zQ$AngLuZ+sGUragqm4bIae/W+ior0TWxYFFaTx8CulqtY".to_string();
    let user_authentication = UserAuthentication {
        user_id: 1i64,
        password_hash,
    };

    assert!(verify_password(password, &user_authentication).is_ok());
    assert!(verify_password(b"incorrect password", &user_authentication).is_err());
}

#[test]
fn password_hashed_with_argon2_should_be_verified() {
    let password = b"87654321";
    let password_hash =
        "$argon2id$v=19$m=4096,t=3,p=1$ycK5lJ4xmFBnaJ51M1j1eA$kU3UlNiSc3JDbl48TCj7JBDKmrT92DOUAgo4Yq0+nMw".to_string();
    let user_authentication = UserAuthentication {
        user_id: 1i64,
        password_hash,
    };

    assert!(verify_password(password, &user_authentication).is_ok());
    assert!(verify_password(b"incorrect password", &user_authentication).is_err());
}
