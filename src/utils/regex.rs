use regex::Regex;

pub fn validate_email_address(email_address_to_be_checked: &str) -> bool {
    let email_regex = Regex::new(r"^([a-z\d_+]([a-z\d_+.]*[a-z\d_+])?)@([a-z\d]+([\-.][a-z\d]+)*\.[a-z]{2,6})").unwrap();

    email_regex.is_match(email_address_to_be_checked)
}

#[cfg(test)]
mod tests {
    use crate::utils::regex::validate_email_address;

    #[test]
    fn validate_email_address_test() {
        assert!(!validate_email_address("test"));

        assert!(!validate_email_address("test@"));

        assert!(!validate_email_address("test@torrust"));

        assert!(!validate_email_address("test@torrust."));

        assert!(!validate_email_address("test@."));

        assert!(!validate_email_address("test@.com"));

        assert!(validate_email_address("test@torrust.com"));

        assert!(validate_email_address("t@torrust.org"))
    }
}
