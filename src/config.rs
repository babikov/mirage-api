use crate::error::Error;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct OpenApi {
    #[allow(dead_code)]
    pub openapi: String,
    pub info: Info,

    #[serde(default)]
    pub paths: HashMap<String, PathItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PathItem {
    #[serde(default)]
    pub get: Option<Operation>,

    #[serde(default)]
    pub post: Option<Operation>,

    #[serde(default)]
    pub put: Option<Operation>,

    #[serde(default)]
    pub delete: Option<Operation>,

    #[serde(default)]
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Operation {
    #[serde(default)]
    #[allow(dead_code)]
    pub summary: Option<String>,

    #[serde(default)]
    pub responses: HashMap<String, Response>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    #[serde(default)]
    #[allow(dead_code)]
    pub description: Option<String>,

    #[serde(default)]
    pub content: HashMap<String, MediaType>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MediaType {
    #[serde(default)]
    pub example: Option<Value>,
}

pub fn load(path: &str) -> Result<OpenApi, Error> {
    let data = std::fs::read_to_string(path)?;
    let spec: OpenApi = serde_yaml::from_str(&data)?;
    Ok(spec)
}
