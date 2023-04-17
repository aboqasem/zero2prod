use std::fmt::Formatter;

use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

pub async fn subscribe(
    data: web::Form<SubscriptionData>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let insertion = sqlx::query!(
        r#"
        INSERT INTO subscriptions
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        data.email,
        data.name,
        chrono::Utc::now()
    )
    .execute(pool.get_ref())
    .await;

    match insertion {
        Ok(_) => HttpResponse::Created().json(SubscriptionResponse {
            message: "Subscribed!".to_string(),
        }),
        Err(error) => {
            eprintln!("Failed to subscribe: {}", error);

            HttpResponse::InternalServerError().json(SubscriptionResponse {
                message: "Failed to subscribe".to_string(),
            })
        }
    }
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
