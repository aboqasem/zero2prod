use std::borrow::Cow;

pub static SEND_PATH: &str = "/v3/mail/send";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailSendBody<'a> {
    pub personalizations: Vec<Personalization<'a>>,
    pub from: From<'a>,
    pub subject: &'a str,
    pub content: Vec<Content<'a>>,
    pub mail_settings: MailSettings,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Personalization<'a> {
    #[serde(borrow)]
    pub to: Vec<To<'a>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct To<'a> {
    pub email: &'a str,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct From<'a> {
    pub email: &'a str,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Content<'a> {
    #[serde(rename = "type")]
    pub mime_type: &'a str,
    pub value: Cow<'a, str>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailSettings {
    pub sandbox_mode: SandboxMode,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxMode {
    pub enable: bool,
}
