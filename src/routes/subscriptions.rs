use std::fmt::Formatter;

use actix_web::{web, HttpResponse, Responder};

pub async fn subscribe(data: web::Form<SubscriptionData>) -> impl Responder {
    HttpResponse::Created().json(SubscriptionResponse {
        message: format!("Received {data}."),
    })
}

#[derive(serde::Deserialize)]
pub struct SubscriptionData {
    name: String,
    email: String,
}

impl std::fmt::Display for SubscriptionData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ name: {}, email: {} }}", self.name, self.email)
    }
}

#[derive(serde::Serialize)]
pub struct SubscriptionResponse {
    message: String,
}
