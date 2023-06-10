extern crate zero2prod;

use crate::utils::spawn_server;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;

use zero2prod::domain::SubscriptionStatus;

#[tokio::test]
async fn subscribe_with_valid_data_should_create_subscription() {
    let (address, pool) = spawn_server().await;

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

    let body = format!("name={name}&email={email}");
    let res = client
        .post(format!("{address}/subscriptions"))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to POST /subscriptions");

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
    let (address, _) = spawn_server().await;

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
        let res = client
            .post(format!("{address}/subscriptions"))
            .body(body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to POST /subscriptions");

        assert_eq!(
            400,
            res.status().as_u16(),
            "Should fail with 400 when body is {desc}"
        );
    }
}
