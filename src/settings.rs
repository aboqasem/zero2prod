use std::env;

use config::{Config, Environment, File};
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;

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
}

#[derive(serde::Deserialize)]
#[allow(unused)]
pub struct AppSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
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
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(self.password.expose_secret())
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.name)
    }
}
