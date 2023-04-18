use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::settings::SETTINGS;
use zero2prod::startup::run_server;

pub type Address = String;

pub async fn spawn_server() -> (Address, PgPool) {
    let pool = create_random_database().await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = run_server(listener, &pool).expect("Failed to start server");

    tokio::spawn(server);

    (address, pool)
}

pub async fn create_random_database() -> PgPool {
    let url_without_db_name = &SETTINGS.database.url_without_db_name();

    let mut conn = PgConnection::connect(url_without_db_name)
        .await
        .expect("Failed to connect to Postgres");

    let db_name = Uuid::new_v4().to_string();
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(format!("{}/{}", url_without_db_name, db_name).as_str())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
