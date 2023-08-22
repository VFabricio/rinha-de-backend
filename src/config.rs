use anyhow::Result;
use config::{Config as ConfigModule, Environment as Env, File};
use serde::Deserialize;
use std::{env, net::IpAddr};

const ENVIRONMENT_VARIABLE: &str = "APP_ENVIRONMENT";
const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug)]
enum Environment {
    Local,
    Production,
}

impl From<String> for Environment {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "local" => Environment::Local,
            "production" => Environment::Production,
            _ => Environment::Local,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Database {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub maxconnections: u32,
}

impl Database {
    pub fn get_connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }

    pub fn get_connection_string_without_database(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}",
            self.user, self.password, self.host, self.port
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Honeycomb {
    pub apikey: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Server {
    pub ip: IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database: Database,
    pub honeycomb: Option<Honeycomb>,
    pub server: Server,
}

pub fn get_config() -> Result<Config> {
    let environment: Environment = env::var(ENVIRONMENT_VARIABLE)
        .unwrap_or("local".into())
        .into();

    let environment_config = ConfigModule::builder().add_source(Env::default().separator("_"));

    let config_builder = match environment {
        Environment::Local => environment_config.add_source(File::with_name(SETTINGS_FILE)),
        _ => environment_config,
    };

    let config = config_builder.build()?;
    let settings = config.try_deserialize()?;
    Ok(settings)
}
