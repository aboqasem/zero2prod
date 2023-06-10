use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::domain::{RawSubscriber, Subscriber, SubscriptionStatus};

#[tracing::instrument(
name = "Adding new subscriber",
skip(subscriber, pool),
fields(
subscriber.email,
subscriber.name,
),
)]
pub async fn subscribe(
    subscriber: web::Form<RawSubscriber>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let subscriber: Subscriber = match subscriber.0.try_into() {
        Ok(s) => s,
        Err(e) => return HttpResponse::BadRequest().json(SubscriptionResponse { message: e }),
    };

    match insert_subscriber(&subscriber, &pool).await {
        Ok(_) => HttpResponse::Created().json(SubscriptionResponse {
            message: "Subscribed!".into(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(SubscriptionResponse {
            message: "Failed to subscribe".into(),
        }),
    }
}

#[tracing::instrument(name = "Inserting subscriber to DB", skip(subscriber, pool))]
async fn insert_subscriber(subscriber: &Subscriber, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions
        VALUES ($1, $2, $3, $4, $5)
        "#,
        uuid::Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        chrono::Utc::now(),
        SubscriptionStatus::Confirmed as _
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert subscriber: {}", e);
        e
    })?;

    Ok(())
}

#[derive(serde::Serialize)]
pub struct SubscriptionResponse {
    message: String,
}
