extern crate zero2prod;

use claims::assert_ge;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use reqwest::{Method, StatusCode};
use wiremock::matchers::{header, header_regex, method, path};
use wiremock::{Mock, ResponseTemplate};

use zero2prod::domain::SubscriptionStatus;
use zero2prod::email::send_grid;
use zero2prod::startup::SUBSCRIPTIONS_PATH;

use crate::utils::{spawn_server, App, links};

async fn post_to_subscriptions(client: &reqwest::Client, address: &reqwest::Url, body: String) -> reqwest::Response {
    client
        .post(format!("{address}{SUBSCRIPTIONS_PATH}"))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .unwrap_or_else(|_| panic!("Failed to POST {SUBSCRIPTIONS_PATH}"))
}

#[tokio::test]
async fn subscribe_with_valid_data_should_create_pending_subscription() {
    // Given
    let App {
        address,
        pool,
        email_server,
    } = spawn_server().await;
    let client = reqwest::Client::new();

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    sqlx::query!(
        "SELECT email, name FROM subscriptions WHERE email = $1",
        email
    )
        .fetch_one(&pool)
        .await
        .expect_err("Should not find a subscription with this email");

    Mock::given(method(Method::POST))
        .and(path(send_grid::SEND_PATH))
        .and(header_regex("Authorization", r"Bearer \w+"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&email_server)
        .await;

    let body = format!("name={name}&email={email}");

    // When
    let res = post_to_subscriptions(&client, &address, body).await;

    // Then
    assert_eq!(StatusCode::CREATED, res.status());

    let saved_subscription = sqlx::query!(
        r#"
        SELECT email, name, status as "status: SubscriptionStatus"
        FROM subscriptions WHERE email = $1
        "#,
        email
    )
        .fetch_one(&pool)
        .await
        .expect("Should find a subscription with this email");

    assert_eq!(saved_subscription.email, email);
    assert_eq!(saved_subscription.name, name);
    assert_eq!(saved_subscription.status, SubscriptionStatus::PendingConfirmation);
}

#[tokio::test]
async fn subscribe_should_send_confirmation_email() {
    // Given
    let App { address, email_server, .. } = spawn_server().await;
    let client = reqwest::Client::new();

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = format!("name={name}&email={email}");

    Mock::given(method(Method::POST))
        .and(path(send_grid::SEND_PATH))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&email_server)
        .await;

    // When
    let _ = post_to_subscriptions(&client, &address, body).await;

    // Then
    let email_request = &email_server.received_requests().await.unwrap()[0];
    let email_body: send_grid::MailSendBody = serde_json::from_slice(&email_request.body).unwrap();
    let links = links(&email_body.content[0].value);

    assert_ge!(links.len(), 1);
}

#[tokio::test]
async fn subscribe_with_invalid_data_should_fail() {
    // Given
    let App { address, .. } = spawn_server().await;
    let client = reqwest::Client::new();

    let name: String = Name().fake();
    let email: String = SafeEmail().fake();

    let invalid_bodies = vec![
        (format!("name={name}"), "missing the email"),
        (format!("email={email}"), "missing the name"),
        ("".to_string(), "missing both of email and name"),
        (
            "name=&email=".to_string(),
            "both of email and name are blank",
        ),
        (
            format!("name=;DROP TABLE subscriptions;--&email={email}"),
            "has forbidden characters",
        ),
    ];

    for (body, desc) in invalid_bodies {
        // When
        let res = post_to_subscriptions(&client, &address, body).await;

        // Then
        assert_eq!(
            StatusCode::BAD_REQUEST,
            res.status(),
            "Should fail with BAD_REQUEST when body is {desc}"
        );
    }
}
