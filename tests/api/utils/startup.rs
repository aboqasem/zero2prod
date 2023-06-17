use std::net::TcpListener;
use std::sync::Once;
use std::time::Duration;

use fake::faker::internet::en::SafeEmail;
use fake::{Fake, Faker};
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

use zero2prod::domain::EmailAddress;
use zero2prod::email::EmailClient;
use zero2prod::settings::{AppBaseUrl, SETTINGS};
use zero2prod::startup::run_server;
use zero2prod::telemetry::{build_subscriber, register_global_subscriber};

pub struct App {
    pub address: reqwest::Url,
    pub pool: PgPool,
    pub email_server: MockServer,
}

static INIT_TELEMETRY: Once = Once::new();

pub async fn spawn_server() -> App {
    INIT_TELEMETRY.call_once(|| {
        let subscriber = build_subscriber("test".into(), "info");

        register_global_subscriber(subscriber);
    });

    let pool = create_random_database().await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let address =
        reqwest::Url::parse(&format!("http://{}", listener.local_addr().unwrap())).unwrap();

    let email_server = MockServer::start().await;
    let email_client = EmailClient::new(
        email_server.uri(),
        Secret::new(Faker.fake()),
        EmailAddress::parse(SafeEmail().fake()).unwrap(),
        Duration::from_millis(200),
        true,
    );

    let server = run_server(listener, &pool, email_client, AppBaseUrl(address.clone()))
        .expect("Failed to start server");

    tokio::spawn(server);

    App {
        address,
        pool,
        email_server,
    }
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
