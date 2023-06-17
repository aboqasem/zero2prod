use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
#[allow(dead_code)]
pub struct ConfirmSubscriptionParameters {
    token: String,
}

#[tracing::instrument(name = "Confirm subscription", skip_all)]
pub async fn confirm_subscription(
    _params: web::Query<ConfirmSubscriptionParameters>,
    _pool: web::Data<PgPool>,
) -> impl Responder {
    HttpResponse::InternalServerError().finish()
}
