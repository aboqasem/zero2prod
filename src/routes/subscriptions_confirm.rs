use crate::domain::SubscriptionStatus;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct ConfirmSubscriptionParameters {
    token: String,
}

#[tracing::instrument(name = "Confirm subscription", skip_all)]
pub async fn confirm_subscription(
    params: web::Query<ConfirmSubscriptionParameters>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let internal_server_error = || HttpResponse::InternalServerError().finish();

    let mut transaction = match pool.begin().await {
        Ok(t) => t,
        Err(_) => return internal_server_error(),
    };

    let subscription_token = &params.token;
    let subscription_id =
        match get_subscription_id_of_subscription_token(subscription_token, &mut transaction).await
        {
            Ok(v) => match v {
                None => return HttpResponse::Unauthorized().finish(),
                Some(v) => v,
            },
            Err(_) => return internal_server_error(),
        };

    if update_subscription_status(
        &subscription_id,
        &SubscriptionStatus::Confirmed,
        &mut transaction,
    )
    .await
    .is_err()
    {
        return internal_server_error();
    }

    if transaction.commit().await.is_err() {
        return internal_server_error();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(name = "Get subscription id of subscription token", skip_all)]
async fn get_subscription_id_of_subscription_token(
    token: &str,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Option<Uuid>, sqlx::Error> {
    let subscription_token = sqlx::query!(
        "SELECT subscription_id FROM subscription_tokens WHERE id = $1",
        token
    )
    .fetch_optional(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch subscription id: {}", e);
        e
    })?;

    Ok(subscription_token.map(|v| v.subscription_id))
}

#[tracing::instrument(name = "Update subscription status", skip_all)]
async fn update_subscription_status(
    subscription_id: &Uuid,
    to_status: &SubscriptionStatus,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE subscriptions SET status = $1 WHERE id = $2",
        to_status as _,
        subscription_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update subscription status: {}", e);
        e
    })?;

    Ok(())
}
