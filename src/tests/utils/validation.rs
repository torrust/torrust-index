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
