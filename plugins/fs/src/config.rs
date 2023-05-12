use serde::Deserialize;
use tauri::utils::config::FsAllowlistScope;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub scope: FsAllowlistScope,
}
