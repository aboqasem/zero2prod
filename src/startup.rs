use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::email::EmailClient;
use crate::routes::{confirm_subscription, health_check, subscribe};
use crate::settings::AppBaseUrl;

pub static HEALTH_PATH: &str = "health";
pub static SUBSCRIPTIONS_PATH: &str = "subscriptions";
pub static SUBSCRIPTIONS_CONFIRM_PATH: &str = "subscriptions/confirm";

pub fn run_server(
    listener: TcpListener,
    pool: &PgPool,
    email_client: EmailClient,
    app_base_url: AppBaseUrl,
) -> Result<Server, std::io::Error> {
    let server = {
        let pool = web::Data::new(pool.clone());
        let email_client = web::Data::new(email_client);
        let app_base_url = web::Data::new(app_base_url);

        HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route(HEALTH_PATH, web::get().to(health_check))
                .route(SUBSCRIPTIONS_PATH, web::post().to(subscribe))
                .route(
                    SUBSCRIPTIONS_CONFIRM_PATH,
                    web::get().to(confirm_subscription),
                )
                .app_data(pool.clone())
                .app_data(email_client.clone())
                .app_data(app_base_url.clone())
        })
        .listen(listener)?
        .run()
    };

    Ok(server)
}
