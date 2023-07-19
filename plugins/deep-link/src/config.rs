#![allow(dead_code)]

#[derive(serde::Deserialize)]
pub struct Config {
    android: Vec<AndroidConfig>,
}

#[derive(serde::Deserialize)]
pub struct AndroidConfig {
    domain: String,
    #[serde(rename = "pathPrefix")]
    path_prefix: Option<String>,
}
