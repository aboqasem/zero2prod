use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::settings::SETTINGS;
use zero2prod::startup::{migrate_db, run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPool::connect(&SETTINGS.database.url())
        .await
        .expect("Failed to connect to Postgres");

    migrate_db(&pool).await;

    let port = SETTINGS.port;
    run(TcpListener::bind(format!("127.0.0.1:{port}"))
        .unwrap_or_else(|_| panic!("Failed to bind to port {port}")))?
    .await
}
