use std::fmt::Write;
use std::num::ParseIntError;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 * bytes.len());

    for byte in bytes {
        write!(s, "{:02X}", byte).unwrap();
    }

    s
}

pub fn hex_to_bytes(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
