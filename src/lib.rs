use std::fmt::Formatter;
use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(serde::Deserialize)]
struct SubscriptionData {
    name: String,
    email: String,
}

impl std::fmt::Display for SubscriptionData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ name: {}, email: {} }}", self.name, self.email)
    }
}

#[derive(serde::Serialize)]
struct SubscriptionResponse {
    message: String,
}

async fn subscribe(data: web::Form<SubscriptionData>) -> impl Responder {
    HttpResponse::Created().json(SubscriptionResponse {
        message: format!("Received {data}."),
    })
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
