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
fn domain_has_extension(domain: &str) -> bool {
    if domain.len() < MIN_DOMAIN_LENGTH {
        return false;
    }

    Regex::new(r".*\..*").expect("Invalid regex").is_match(domain)
}

#[cfg(test)]
mod tests {

    mod for_email_validation {
        use crate::utils::validation::validate_email_address;

        #[test]
        fn it_should_accept_valid_email_addresses() {
            assert!(validate_email_address("test@torrust.com"));
            assert!(validate_email_address("t@torrust.org"));
        }

        #[test]
        fn it_should_not_accept_invalid_email_addresses() {
            assert!(!validate_email_address("test"));
            assert!(!validate_email_address("test@"));
            assert!(!validate_email_address("test@torrust."));
            assert!(!validate_email_address("test@."));
            assert!(!validate_email_address("test@.com"));

            // Notice that local domain name with no TLD are valid,
            // although ICANN highly discourages dotless email addresses
            assert!(!validate_email_address("test@torrust"));
        }
    }

    mod for_domain_validation {
        use crate::utils::validation::domain_has_extension;

        #[test]
        fn it_should_accept_valid_domain_with_extension() {
            assert!(domain_has_extension("a.io"));
            assert!(domain_has_extension("a.com"));
        }

        #[test]
        fn it_should_not_accept_dotless_domains() {
            assert!(!domain_has_extension(""));
            assert!(!domain_has_extension("."));
            assert!(!domain_has_extension("a."));
        }
    }
}
