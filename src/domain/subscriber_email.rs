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

    #[test]
    fn proper_email_should_succeed() {
        let email = "something.else@email.com".to_string();
        assert_ok!(SubscriberEmail::parse(email));
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
