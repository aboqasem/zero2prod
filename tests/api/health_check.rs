use zero2prod::startup::HEALTH_PATH;

use crate::utils::{spawn_server, App};

#[tokio::test]
async fn health_check_works() {
    let App { address, .. } = spawn_server().await;

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}{HEALTH_PATH}"))
        .send()
        .await
        .unwrap_or_else(|_| panic!("Failed to GET {HEALTH_PATH}"));

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}
