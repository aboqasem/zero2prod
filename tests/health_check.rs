use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_server();

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/health"))
        .send()
        .await
        .expect("Failed to GET /health");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}

fn spawn_server() -> String {
    let host = "127.0.0.1";
    let listener = TcpListener::bind(format!("{host}:0")).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::run(listener).expect("Failed to bind address");
    tokio::spawn(server);

    format!("http://{host}:{port}")
}
