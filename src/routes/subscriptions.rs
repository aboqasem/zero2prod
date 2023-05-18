use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(data, pool),
    fields(
        subscriber.email = data.email,
        subscriber.name = data.name,
    ),
)]
pub async fn subscribe(
    data: web::Form<SubscriptionData>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    match insert_subscriber(&data, &pool).await {
        Ok(_) => HttpResponse::Created().json(SubscriptionResponse {
            message: "Subscribed!".into(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(SubscriptionResponse {
            message: "Failed to subscribe".into(),
        }),
    }
}

#[tracing::instrument(name = "Inserting subscriber to DB", skip(data, pool))]
async fn insert_subscriber(data: &SubscriptionData, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions
        VALUES ($1, $2, $3, $4)
        "#,
        uuid::Uuid::new_v4(),
        data.email,
        data.name,
        chrono::Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert subscriber: {}", e);
        e
    })?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct SubscriptionData {
    name: String,
    email: String,
}

#[derive(serde::Serialize)]
pub struct SubscriptionResponse {
    message: String,
}
