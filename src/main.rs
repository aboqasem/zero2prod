use std::net::TcpListener;

use zero2prod::settings::SETTINGS;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = SETTINGS.port;
    run(TcpListener::bind(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|_| panic!("Failed to bind to port {port}")))?
    .await
}
