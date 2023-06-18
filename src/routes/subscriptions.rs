use actix_web::{web, HttpResponse, Responder};
use rand::distributions;
use rand::{thread_rng, Rng};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{RawSubscriber, Subscriber, SubscriptionStatus};
use crate::email::{EmailClient, EmailData};
use crate::settings::AppBaseUrl;
use crate::startup::SUBSCRIPTIONS_CONFIRM_PATH;

#[tracing::instrument(
    name = "Adding new subscriber",
    skip_all,
    fields(
        subscriber.email,
        subscriber.name,
    ),
)]
pub async fn subscribe(
    subscriber: web::Form<RawSubscriber>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<AppBaseUrl>,
) -> impl Responder {
    let subscriber: Subscriber = match subscriber.0.try_into() {
        Ok(s) => s,
        Err(e) => return HttpResponse::BadRequest().json(SubscriptionResponse { message: e }),
    };

    let internal_server_error = || {
        HttpResponse::InternalServerError().json(SubscriptionResponse {
            message: "Failed to subscribe".into(),
        })
    };

    let subscriber_id = match insert_subscriber(&subscriber, &pool).await {
        Ok(id) => id,
        Err(_) => return internal_server_error(),
    };

    let subscription_token = match insert_random_subscription_token(&subscriber_id, &pool).await {
        Ok(token) => token,
        Err(_) => return internal_server_error(),
    };

    if send_confirmation_email(
        &email_client,
        &subscriber,
        &app_base_url.as_ref().0,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return internal_server_error();
    };

    HttpResponse::Created().json(SubscriptionResponse {
        message: "Subscribed!".into(),
    })
}

#[tracing::instrument(name = "Inserting subscriber to DB", skip_all)]
async fn insert_subscriber(subscriber: &Subscriber, pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions
        VALUES ($1, $2, $3, $4, $5)
        "#,
        subscriber_id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        chrono::Utc::now(),
        SubscriptionStatus::PendingConfirmation as _
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert subscriber: {}", e);
        e
    })?;

    Ok(subscriber_id)
}

#[tracing::instrument(name = "Inserting random subscription token to DB", skip_all)]
async fn insert_random_subscription_token(
    subscription_id: &Uuid,
    pool: &PgPool,
) -> Result<String, sqlx::Error> {
    let token = gen_random_string(25);
    sqlx::query!(
        r#"
        INSERT INTO subscription_tokens
        VALUES ($1, $2)
        "#,
        token,
        subscription_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert subscription token: {}", e);
        e
    })?;

    Ok(token)
}

#[tracing::instrument(name = "Sending confirmation email", skip_all)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber: &Subscriber,
    base_url: &reqwest::Url,
    token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!("{base_url}{SUBSCRIPTIONS_CONFIRM_PATH}?token={token}");

    let email_data = EmailData {
        to: subscriber.email.clone(),
        subject: "Welcome!".into(),
        content: format!(
            r#"Welcome to my newsletter, {}!<br>
<br>
You may <a href="{confirmation_link}">confirm your subscription by clicking here!</a>
"#,
            subscriber.name.as_ref()
        ),
        content_type: "text/html".into(),
    };

    email_client.send(&email_data).await
}

fn gen_random_string(len: usize) -> String {
    let mut rng = thread_rng();

    std::iter::repeat_with(|| rng.sample(distributions::Alphanumeric))
        .map(char::from)
        .take(len)
        .collect()
}

#[derive(serde::Serialize)]
pub struct SubscriptionResponse {
    message: String,
}
