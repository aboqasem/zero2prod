use crate::subscriptions::{base_send_grid_send_endpoint_mock, post_valid_body_to_subscriptions};
use reqwest::StatusCode;
use zero2prod::domain::SubscriptionStatus;
use zero2prod::email::send_grid;
use zero2prod::startup::SUBSCRIPTIONS_CONFIRM_PATH;

use crate::utils::{links, spawn_server, App};

#[tokio::test]
async fn confirmation_without_token_fails() {
    let App { address, .. } = spawn_server().await;

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}{SUBSCRIPTIONS_CONFIRM_PATH}"))
        .send()
        .await
        .unwrap_or_else(|_| panic!("Failed to GET {SUBSCRIPTIONS_CONFIRM_PATH}"));

    assert_eq!(StatusCode::BAD_REQUEST, res.status());
}

#[tokio::test]
async fn confirmation_received_from_subscribe_works() {
    // Given
    let App {
        address,
        email_server,
        pool,
    } = spawn_server().await;

    let client = reqwest::Client::new();

    base_send_grid_send_endpoint_mock()
        .expect(1)
        .mount(&email_server)
        .await;

    // When
    let (_, subscriber) = post_valid_body_to_subscriptions(&client, &address).await;

    let email_request = &email_server.received_requests().await.unwrap()[0];
    let email_body: send_grid::MailSendBody = serde_json::from_slice(&email_request.body).unwrap();
    let email_links = links(&email_body.content[0].value);

    let mut confirmation_link = reqwest::Url::parse(email_links[0].as_str()).unwrap();
    confirmation_link.set_port(address.port()).unwrap();

    let res = client
        .get(confirmation_link.as_str())
        .send()
        .await
        .unwrap_or_else(|_| panic!("Failed to GET {confirmation_link}"));

    // Then
    assert_eq!(StatusCode::OK, res.status());

    let user = sqlx::query!(
        r#"SELECT status as "status: SubscriptionStatus" FROM subscriptions WHERE email = $1"#,
        subscriber.email.as_ref()
    )
    .fetch_one(&pool)
    .await
    .expect("Should find a subscription with this email");

    assert_eq!(user.status, SubscriptionStatus::Confirmed);
}
