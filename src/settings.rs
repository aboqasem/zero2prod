use std::env;

use config::{Config, Environment, File};
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

// use crate::domain::EmailAddress;

pub static SETTINGS: Lazy<Settings> = Lazy::new(|| {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    Config::builder()
        // Start off by merging in the default
        .add_source(File::with_name("settings"))
        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        .add_source(File::with_name(&format!("settings.{run_mode}")).required(false))
        // Add in a local configuration file
        .add_source(File::with_name("settings.local").required(false))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APP__PORT=5001 would set `Settings.app.port`
        .add_source(
            Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .expect("Failed to build config")
        .try_deserialize()
        .expect("Failed to deserialize config")
});

#[derive(serde::Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub app: AppSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Clone)]
pub struct AppBaseUrl(pub reqwest::Url);

pub fn deserialize_url_from_string<'de, D>(deserializer: D) -> Result<reqwest::Url, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;

    reqwest::Url::parse(str.as_str()).map_err(serde::de::Error::custom)
}

pub fn deserialize_app_base_url_from_string<'de, D>(deserializer: D) -> Result<AppBaseUrl, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    Ok(AppBaseUrl(deserialize_url_from_string(deserializer)?))
}

#[derive(serde::Deserialize)]
#[allow(unused)]
pub struct AppSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    #[serde(deserialize_with = "deserialize_app_base_url_from_string")]
    pub base_url: AppBaseUrl,
}

#[derive(serde::Deserialize)]
#[allow(unused)]
pub struct DatabaseSettings {
    pub name: String,
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
            .ssl_mode(if self.require_ssl {
                PgSslMode::Require
            } else {
                PgSslMode::Prefer
            })
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.name)
    }
}

#[derive(serde::Deserialize)]
#[allow(unused)]
pub struct EmailClientSettings {
    pub api_key: Secret<String>,
    pub base_url: String,
    pub sender: crate::domain::EmailAddress,
    pub sandbox: bool,
    pub timeout_millis: u64,
}
