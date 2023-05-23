use std::net::TcpListener;
use std::sync::Once;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use zero2prod::settings::SETTINGS;
use zero2prod::startup::run_server;
use zero2prod::telemetry::{build_subscriber, register_global_subscriber};

pub type Address = String;

static INIT_TELEMETRY: Once = Once::new();

pub async fn spawn_server() -> (Address, PgPool) {
    INIT_TELEMETRY.call_once(|| {
        let subscriber = build_subscriber("test".into(), "info");

        register_global_subscriber(subscriber);
    });

    let pool = create_random_database().await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = run_server(listener, &pool).expect("Failed to start server");

    tokio::spawn(server);

    (address, pool)
}

pub async fn create_random_database() -> PgPool {
    let mut conn = PgConnection::connect_with(&SETTINGS.database.without_db())
        .await
        .expect("Failed to connect to Postgres");

    let db_name = Uuid::new_v4().to_string();
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect_with(SETTINGS.database.without_db().database(&db_name))
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
