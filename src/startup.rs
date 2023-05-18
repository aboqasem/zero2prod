use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::{health_check, subscribe};

pub fn run_server(listener: TcpListener, pool: &PgPool) -> Result<Server, std::io::Error> {
    let server = {
        let pool = web::Data::new(pool.clone());

        HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .app_data(pool.clone())
        })
        .listen(listener)?
        .run()
    };

    Ok(server)
}
