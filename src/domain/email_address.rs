#[derive(Clone, Debug, serde::Deserialize)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse(raw_email: String) -> Result<Self, String> {
        if !validator::validate_email(&raw_email) {
            return Err("invalid email address".into());
        }

        Ok(Self(raw_email))
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Gen;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            Self(SafeEmail().fake_with_rng(&mut rng))
        }
    }

    #[test]
    fn an_empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(EmailAddress::parse(email));
    }

    #[test]
    fn an_email_without_at_symbol_is_rejected() {
        let email = "zouabi.com".to_string();
        assert_err!(EmailAddress::parse(email));
    }

    #[quickcheck_macros::quickcheck]
    fn a_valid_email_is_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        EmailAddress::parse(valid_email.0).is_ok()
    }
}
