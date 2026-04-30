//! # Auth-keypair integration tests
//!
//! | Test                                    | What it covers                            |
//! |-----------------------------------------|-------------------------------------------|
//! | `generated_keypair_round_trips_as_json` | JSON output deserialises back correctly   |
//! | `output_contains_valid_pem_keys`        | PEM keys parse as RSA PKCS#8 / SPKI      |
//! | `successive_calls_produce_distinct_keys` | No hardcoded/cached key material         |

use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use torrust_index_auth_keypair::{KeypairOutput, generate_keypair};

#[test]
fn generated_keypair_round_trips_as_json() {
    let output = generate_keypair().unwrap();
    let json = serde_json::to_string(&output).unwrap();
    let parsed: KeypairOutput = serde_json::from_str(&json).unwrap();
    assert!(!parsed.private_key_pem.is_empty());
    assert!(!parsed.public_key_pem.is_empty());
}

#[test]
fn output_contains_valid_pem_keys() {
    let output = generate_keypair().unwrap();

    rsa::RsaPrivateKey::from_pkcs8_pem(&output.private_key_pem).expect("private_key_pem should be valid PKCS#8");
    rsa::RsaPublicKey::from_public_key_pem(&output.public_key_pem).expect("public_key_pem should be valid SPKI");
}

#[test]
fn successive_calls_produce_distinct_keys() {
    let kp1 = generate_keypair().unwrap();
    let kp2 = generate_keypair().unwrap();

    assert_ne!(
        kp1.private_key_pem, kp2.private_key_pem,
        "private keys should differ between calls"
    );
    assert_ne!(
        kp1.public_key_pem, kp2.public_key_pem,
        "public keys should differ between calls"
    );
}
