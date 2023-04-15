use std::env;

use config::{Config, File};
use once_cell::sync::Lazy;

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
        .build()
        .expect("Failed to build config")
        .try_deserialize()
        .expect("Failed to deserialize config")
});

#[derive(serde::Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub port: u16,
}
