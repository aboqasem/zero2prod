use crate::domain::EmailAddress;

pub struct EmailData {
    pub to: EmailAddress,
    pub subject: String,
    pub content: String,
    pub content_type: String,
}
