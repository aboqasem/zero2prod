use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::settings::SETTINGS;
use zero2prod::startup::run_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect(&SETTINGS.database.url())
        .await
        .expect("Failed to connect to Postgres");

    let listener = {
        let port = SETTINGS.port;
        let address = format!("127.0.0.1:{port}");
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Failed to bind to port {port}"))
    };

    run_server(listener, &pool)?.await
}
