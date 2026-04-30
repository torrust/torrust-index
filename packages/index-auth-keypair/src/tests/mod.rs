//! # Auth-keypair tests
//!
//! | Test                                  | What it covers                          |
//! |---------------------------------------|-----------------------------------------|
//! | `generated_json_round_trips`          | JSON output deserialises back           |
//! | `private_pem_parses`                  | Private key PEM is valid PKCS#8         |
//! | `public_pem_parses`                   | Public key PEM is valid SPKI            |

use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};

#[test]
fn generated_json_round_trips() {
    let output = super::generate_keypair().unwrap();
    let json = serde_json::to_string(&output).unwrap();
    let parsed: super::KeypairOutput = serde_json::from_str(&json).unwrap();
    assert!(!parsed.private_key_pem.is_empty());
    assert!(!parsed.public_key_pem.is_empty());
}

#[test]
fn private_pem_parses() {
    let output = super::generate_keypair().unwrap();
    rsa::RsaPrivateKey::from_pkcs8_pem(&output.private_key_pem).expect("private key PEM should parse");
}

#[test]
fn public_pem_parses() {
    let output = super::generate_keypair().unwrap();
    rsa::RsaPublicKey::from_public_key_pem(&output.public_key_pem).expect("public key PEM should parse");
}
