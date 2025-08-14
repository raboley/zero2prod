use actix_web::body;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
    // Store the full email endpoint URL
    email_url: reqwest::Url,
}

impl EmailClient {
    pub fn new(
        base_url: reqwest::Url,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Self {
        // This is safe: "email" is a valid path segment that will always parse
        let email_url = base_url
            .join("email")
            .expect("Failed to append /email to base URL");
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
            email_url,
        }
    }
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_body: html_content.to_owned(),
            text_body: text_content.to_owned(),
        };
        self.http_client
            .post(self.email_url.clone())
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?;

        // Setting the body with an object using Serde Json also sets these headers automatically
        // .header("Content-Type", "application/json")
        // .header("Accept", "application/json")

        // .body("")
        // .send()
        // .await;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use reqwest::Url;
    use secrecy::Secret;
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{domain::SubscriberEmail, email_client::EmailClient};

    use wiremock::Request;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let uri = Url::parse(&mock_server.uri()).expect("Failed to parse mock uri");
        let email_client = EmailClient::new(uri, sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
}
