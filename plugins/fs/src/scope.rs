use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Entry {
    pub path: PathBuf,
}
