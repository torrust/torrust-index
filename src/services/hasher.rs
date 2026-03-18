//! Hashing service
use sha1::{Digest, Sha1};

// Calculate the sha1 hash of a string
#[must_use]
pub fn sha1(data: &str) -> String {
    // Create a Sha1 object
    let mut hasher = Sha1::new();

    // Write input message
    hasher.update(data.as_bytes());

    // Read hash digest and consume hasher
    let result = hasher.finalize();

    // Convert the hash (a byte array) to a string of hex characters
    hex::encode(result)
}
