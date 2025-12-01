use serde::Deserialize;
use crate::error::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct RouteConfig {
    pub method: String,
    pub path: String,
}

pub fn load(path: &str) -> Result<Config, Error> {
    let data = std::fs::read_to_string(path)?;
    let cfg: Config = serde_yaml::from_str(&data)?;
    Ok(cfg)
}
