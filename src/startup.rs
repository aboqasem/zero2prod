use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::email::EmailClient;
use crate::routes::{health_check, subscribe};

pub fn run_server(
    listener: TcpListener,
    pool: &PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let server = {
        let pool = web::Data::new(pool.clone());
        let email_client = web::Data::new(email_client);

        HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .app_data(pool.clone())
                .app_data(email_client.clone())
        })
        .listen(listener)?
        .run()
    };

    Ok(server)
}
