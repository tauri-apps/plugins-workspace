// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub kind: ScanKind,
    pub keep_session_alive: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NfcRecord {
    pub format: NFCTypeNameFormat,
    pub kind: Vec<u8>,
    pub id: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum NFCTypeNameFormat {
    Empty = 0,
    NfcWellKnown = 1,
    Media = 2,
    AbsoluteURI = 3,
    NfcExternal = 4,
    Unknown = 5,
    Unchanged = 6,
}

#[derive(Deserialize)]
pub struct NfcTagRecord {
    pub tnf: NFCTypeNameFormat,
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

#[derive(Debug, Default, Serialize)]
pub struct UriFilter {
    scheme: Option<String>,
    host: Option<String>,
    path_prefix: Option<String>,
}

#[derive(Debug)]
pub enum TechKind {
    IsoDep,
    MifareClassic,
    MifareUltralight,
    Ndef,
    NdefFormatable,
    NfcA,
    NfcB,
    NfcBarcode,
    NfcF,
    NfcV,
}

impl Display for TechKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IsoDep => "IsoDep",
                Self::MifareClassic => "MifareClassic",
                Self::MifareUltralight => "MifareUltralight",
                Self::Ndef => "Ndef",
                Self::NdefFormatable => "NdefFormatable",
                Self::NfcA => "NfcA",
                Self::NfcB => "NfcB",
                Self::NfcBarcode => "NfcBarcode",
                Self::NfcF => "NfcF",
                Self::NfcV => "NfcV",
            }
        )
    }
}

impl Serialize for TechKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanKind {
    Ndef {
        mime_type: Option<String>,
        uri: Option<UriFilter>,
        tech_list: Option<Vec<Vec<TechKind>>>,
    },
    Tag {
        mime_type: Option<String>,
        uri: Option<UriFilter>,
    },
}
