extern crate zero2prod;

use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use reqwest::{Method, StatusCode};
use wiremock::matchers::{header, header_regex, method, path};
use wiremock::{Mock, ResponseTemplate};

use zero2prod::domain::SubscriptionStatus;
use zero2prod::email::send_grid;

use crate::utils::{spawn_server, App};

#[tokio::test]
async fn subscribe_with_valid_data_should_create_subscription() {
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
    let res = client
        .post(format!("{address}/subscriptions"))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to POST /subscriptions");

    // Then
    assert_eq!(201, res.status().as_u16());

    let saved_subscription = sqlx::query!(
        r#"SELECT email, name, status as "status: SubscriptionStatus"  FROM subscriptions WHERE email = $1"#,
        email
    )
    .fetch_one(&pool)
    .await
    .expect("Should find a subscription with this email");

    assert_eq!(saved_subscription.email, email);
    assert_eq!(saved_subscription.name, name);
    assert_eq!(saved_subscription.status, SubscriptionStatus::Confirmed);
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
        let res = client
            .post(format!("{address}/subscriptions"))
            .body(body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to POST /subscriptions");

        // Then
        assert_eq!(
            400,
            res.status().as_u16(),
            "Should fail with 400 when body is {desc}"
        );
    }
}
