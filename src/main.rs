use secrecy::ExposeSecret;
use std::net::TcpListener;

use sqlx::PgPool;

use zero2prod::settings::SETTINGS;
use zero2prod::startup::run_server;
use zero2prod::telemetry::{build_subscriber, register_global_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = build_subscriber("zero2prod".into(), "info");
    register_global_subscriber(subscriber);

    let pool = PgPool::connect(SETTINGS.database.url().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let listener = {
        let (host, port) = (&SETTINGS.app.host, SETTINGS.app.port);
        let address = format!("{host}:{port}");
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Failed to bind to port {port}"))
    };

    run_server(listener, &pool)?.await
}
