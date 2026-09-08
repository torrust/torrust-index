//! # Auth-keypair tests
//!
//! | Test                                  | What it covers                          |
//! |---------------------------------------|-----------------------------------------|
//! | `generated_json_round_trips`          | JSON output deserialises back           |
//! | `generated_output_carries_schema`      | Output schema field is stable           |
//! | `private_pem_parses`                  | Private key PEM is valid PKCS#8         |
//! | `public_pem_parses`                   | Public key PEM is valid SPKI            |

use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};

#[test]
fn generated_json_round_trips() {
    let output = super::generate_keypair().unwrap();
    let json = serde_json::to_string(&output).unwrap();
    let parsed: super::KeypairOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema, super::SCHEMA);
    assert_ne!(parsed.private_key_pem, "");
    assert_ne!(parsed.public_key_pem, "");
}

#[test]
fn generated_output_carries_schema() {
    let output = super::generate_keypair().unwrap();
    assert_eq!(output.schema, super::SCHEMA);
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
