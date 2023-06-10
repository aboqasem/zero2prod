use crate::utils::spawn_server;

#[tokio::test]
async fn health_check_works() {
    let (address, _) = spawn_server().await;

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/health"))
        .send()
        .await
        .expect("Failed to GET /health");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}
