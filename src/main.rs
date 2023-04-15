use std::net::TcpListener;

use zero2prod::settings::SETTINGS;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = SETTINGS.port;
    run(TcpListener::bind(format!("127.0.0.1:{port}"))
        .expect(format!("Failed to bind to port {port}").as_str()))?
    .await
}
