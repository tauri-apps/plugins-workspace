// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub kind: ScanKind,
    pub keep_alive: bool,
}

#[derive(Deserialize)]
pub struct NfcTagRecord {
    pub tnf: u8,
    pub kind: Vec<u8>,
    pub id: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Deserialize)]
pub struct NfcTag {
    pub id: String,
    pub kind: String,
    pub records: Vec<NfcTagRecord>,
}

#[derive(Deserialize)]
pub struct ScanResponse {
    pub tag: NfcTag,
}

#[derive(Debug)]
pub enum ScanKind {
    Ndef,
    Tag,
}

impl Display for ScanKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ndef => "ndef",
                Self::Tag => "tag",
            }
        )
    }
}

impl Serialize for ScanKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
