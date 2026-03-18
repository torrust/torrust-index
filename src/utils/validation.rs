use std::str::FromStr;

use email_address::EmailAddress;
use regex::Regex;

const MIN_DOMAIN_LENGTH: usize = 4;

/// Validates an email address.
///
/// # Panics
///
/// It panics if the email address is invalid. This should not happen
/// because the email address is previously validated.
#[must_use]
pub fn validate_email_address(email_address_to_be_checked: &str) -> bool {
    if !EmailAddress::is_valid(email_address_to_be_checked) {
        return false;
    }

    let email = EmailAddress::from_str(email_address_to_be_checked).expect("Invalid email address");

    // We reject anyway the email if it's a dotless domain name.
    domain_has_extension(email.domain())
}

/// Returns true if the string representing a domain has an extension.
///
/// It does not check if the extension is valid.
pub(crate) fn domain_has_extension(domain: &str) -> bool {
    if domain.len() < MIN_DOMAIN_LENGTH {
        return false;
    }

    Regex::new(r".*\..*").expect("Invalid regex").is_match(domain)
}
