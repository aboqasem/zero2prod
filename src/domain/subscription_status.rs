#[derive(sqlx::Type, Debug, PartialEq)]
#[sqlx(type_name = "subscription_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionStatus {
    PendingConfirmation,
    Confirmed,
}
