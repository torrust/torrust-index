//! RSA-2048 key pair generator for Torrust Index JWT authentication.
//!
//! Outputs a JSON object with `schema`, `private_key_pem`, and `public_key_pem`
//! fields to stdout per ADR-T-010. Diagnostics go to stderr via JSON `tracing`.

use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Schema version of the keypair JSON output.
pub const SCHEMA: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct KeypairOutput {
    pub schema: u32,
    pub private_key_pem: String,
    pub public_key_pem: String,
}

/// # Errors
///
/// Returns an error string if RSA key generation or PEM export fails.
pub fn generate_keypair() -> Result<KeypairOutput, String> {
    let mut rng = rsa::rand_core::OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|e| format!("RSA key generation failed: {e}"))?;

    let private_pem = private_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| format!("private key PEM export failed: {e}"))?;

    let public_pem = private_key
        .to_public_key()
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| format!("public key PEM export failed: {e}"))?;

    Ok(KeypairOutput {
        schema: SCHEMA,
        private_key_pem: private_pem.to_string(),
        public_key_pem: public_pem,
    })
}
