use std::borrow::Cow;
use std::time::Duration;

use secrecy::{ExposeSecret, Secret};

use crate::domain::EmailAddress;
use crate::email::{send_grid, EmailData};
use crate::settings::SETTINGS;

pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
    api_key: Secret<String>,
    from: EmailAddress,
    sandbox: bool,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        api_key: Secret<String>,
        from: EmailAddress,
        timeout: Duration,
        sandbox: bool,
    ) -> Self {
        Self {
            http_client: reqwest::Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            from,
            api_key,
            sandbox,
        }
    }

    pub async fn send(&self, email: &EmailData) -> Result<(), reqwest::Error> {
        let body = send_grid::MailSendBody {
            personalizations: vec![send_grid::Personalization {
                to: vec![send_grid::To {
                    email: email.to.as_ref(),
                }],
            }],
            from: send_grid::From {
                email: self.from.as_ref(),
            },
            subject: &email.subject,
            content: vec![send_grid::Content {
                mime_type: &email.content_type,
                value: Cow::from(&email.content),
            }],
            mail_settings: send_grid::MailSettings {
                sandbox_mode: send_grid::SandboxMode {
                    enable: self.sandbox,
                },
            },
        };

        self.http_client
            .post(format!("{}{}", self.base_url, send_grid::SEND_PATH))
            .json(&body)
            .header(
                "Authorization",
                format!("Bearer {token}", token = self.api_key.expose_secret()),
            )
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

impl Default for EmailClient {
    fn default() -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_millis(SETTINGS.email_client.timeout_millis))
                .build()
                .unwrap(),
            from: SETTINGS.email_client.sender.clone(),
            base_url: SETTINGS.email_client.base_url.clone(),
            api_key: SETTINGS.email_client.api_key.clone(),
            sandbox: SETTINGS.email_client.sandbox,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use reqwest::{Method, StatusCode};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_regex, method, path};
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

    use crate::domain::EmailAddress;
    use crate::email::{send_grid, EmailClient, EmailData};

    struct SendEmailBodyMatcher;

    impl Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let parsed_body: Result<send_grid::MailSendBody, _> =
                serde_json::from_slice(&request.body);

            parsed_body.is_ok()
        }
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            Secret::new(Faker.fake()),
            EmailAddress::parse(SafeEmail().fake()).unwrap(),
            Duration::from_millis(200),
            true,
        )
    }

    fn fake_email_data() -> EmailData {
        EmailData {
            to: EmailAddress::parse(SafeEmail().fake()).unwrap(),
            subject: Sentence(1..2).fake(),
            content: Paragraph(1..10).fake(),
            content_type: Sentence(1..2).fake(),
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_valid_request_to_server() {
        // Given
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(method(Method::POST))
            .and(path(send_grid::SEND_PATH))
            .and(header_regex("Authorization", r"Bearer \w+"))
            .and(header("Content-Type", "application/json"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        // When
        let result = email_client.send(&fake_email_data()).await;

        // Then
        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_fails() {
        // Given
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .expect(1)
            .mount(&mock_server)
            .await;

        // When
        let result = email_client.send(&fake_email_data()).await;

        // Then
        assert_err!(result);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Given
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(
                ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .set_delay(Duration::from_secs(60 * 3)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        // When
        let result = email_client.send(&fake_email_data()).await;

        // Then
        assert_err!(result);
    }
}
