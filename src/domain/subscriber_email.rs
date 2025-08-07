use validator::ValidateEmail;

#[derive(Debug)]
struct SubscriberEmail(String);

impl SubscriberEmail {
    fn parse(s: String) -> Result<SubscriberEmail, String> {
        if s.validate_email() {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::{assert_err, assert_ok};

    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    use rand::rngs::StdRng;
    use rand::{random, SeedableRng};

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);

            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        dbg!(&valid_email.0);

        SubscriberEmail::parse(valid_email.0).is_ok()
    }

    #[test]
    fn invalid_email_should_fail() {
        let test_cases = vec![
            ("something.else.email.com", "missing @ symbol"),
            ("", "empty string"),
        ];
        for (email, description) in test_cases {
            assert_err!(
                SubscriberEmail::parse(email.to_string()),
                "Failed! Test case: {}",
                description
            );
        }
    }
}
