use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::settings::SETTINGS;
use zero2prod::startup::run_server;

pub async fn spawn_server() -> (Address, PgPool) {
    let pool = PgPool::connect(&SETTINGS.database.url())
        .await
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = run_server(listener, &pool).expect("Failed to start server");

    tokio::spawn(server);

    (address, pool)
}

pub type Address = String;
