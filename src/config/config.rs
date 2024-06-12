use crate::prelude::Result as AppResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use twelf::{config, Layer};


pub fn load_config(path: PathBuf) -> AppResult<Configuration> {
    let path = path.into();
    // Layer from different sources to build configuration. Order matters!
    let conf = Configuration::with_layers(&[
        Layer::Yaml(path),
        Layer::Env(Some(String::from("APP_"))),
    ])?;

    Ok(conf)
}

#[config]
#[derive(Debug, Clone, Default, Serialize)]
pub struct Configuration {
    pub app: AppConfiguration,
    pub db: DBConfiguration,
    pub tls: TLSConfiguration,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppConfiguration {
    pub name: String,
    pub host: String,
    pub port: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DBConfiguration {
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TLSConfiguration {
    pub cert_path: String,
    pub key_path: String,
}
