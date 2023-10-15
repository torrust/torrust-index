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

#[cfg(test)]
mod tests {
    use crate::services::hasher::sha1;

    #[test]
    fn it_should_hash_an_string() {
        assert_eq!(sha1("hello world"), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
}
