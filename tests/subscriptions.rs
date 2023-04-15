use crate::utils::spawn_server;

mod utils;

#[tokio::test]
async fn subscribe_with_valid_data_is_201() {
    let address = spawn_server();

    let client = reqwest::Client::new();

    let body = "name=Mohammad%20Al%20Zouabi&email=mb.alzouabi%40gmail.com";
    let res = client
        .post(format!("{address}/subscriptions"))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to POST /subscriptions");

    assert_eq!(201, res.status().as_u16());
}

#[tokio::test]
async fn subscribe_with_invalid_data_is_400() {
    let address = spawn_server();

    let client = reqwest::Client::new();

    let invalid_bodies = vec![
        ("name=Mohammad%20Al%20Zouabi", "missing the email"),
        ("email=mb.alzouabi%40gmail.com", "missing the name"),
        ("", "missing both of email and name"),
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
