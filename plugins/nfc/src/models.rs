// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

/// Arguments of the [`Nfc::scan`](crate::Nfc::scan) API.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    /// The kind of scan to perform, which defines how tags are matched.
    pub kind: ScanKind,
    /// Whether the connection to the scanned tag must be kept open after the scan resolves.
    ///
    /// When `true`, a following [`Nfc::write`](crate::Nfc::write) call writes to the tag that was
    /// scanned instead of starting a new session.
    pub keep_session_alive: bool,
}

/// An NDEF record to be written to a tag.
///
/// Use [`NFCTypeNameFormat`] to describe how [`Self::kind`] must be interpreted.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NfcRecord {
    /// The Type Name Format (TNF) of the record.
    pub format: NFCTypeNameFormat,
    /// The record type, interpreted according to [`Self::format`].
    ///
    /// For [`NFCTypeNameFormat::NfcWellKnown`] records this is a Record Type Definition (RTD)
    /// value such as `[0x54]` (`RTD_TEXT`) or `[0x55]` (`RTD_URI`).
    pub kind: Vec<u8>,
    /// The record identifier. Can be empty.
    pub id: Vec<u8>,
    /// The record payload bytes.
    pub payload: Vec<u8>,
}

/// The Type Name Format (TNF) of an NDEF record, which defines how the record type is interpreted.
///
/// Serialized as its numeric value.
#[derive(serde_repr::Deserialize_repr, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum NFCTypeNameFormat {
    /// The record is empty: type, identifier and payload must be empty.
    Empty = 0,
    /// The record type is an NFC Forum well known type, defined by a Record Type Definition (RTD)
    /// such as `RTD_TEXT` (`[0x54]`) or `RTD_URI` (`[0x55]`).
    NfcWellKnown = 1,
    /// The record type is a MIME media type as defined in RFC 2046, e.g. `text/plain`.
    Media = 2,
    /// The record type is an absolute URI as defined in RFC 3986.
    AbsoluteURI = 3,
    /// The record type is an NFC Forum external type, i.e. a type namespaced by its issuer.
    NfcExternal = 4,
    /// The record type is unknown: the type must be empty and the payload interpretation is
    /// left to the application.
    Unknown = 5,
    /// The record is a middle or last chunk of a chunked record and inherits the type of the
    /// first chunk, so its own type must be empty.
    Unchanged = 6,
}

/// An NDEF record read from a scanned tag.
#[derive(Deserialize)]
pub struct NfcTagRecord {
    /// The Type Name Format (TNF) of the record, which defines how [`Self::kind`] is interpreted.
    pub tnf: NFCTypeNameFormat,
    /// The record type bytes.
    pub kind: Vec<u8>,
    /// The record identifier bytes. Can be empty.
    pub id: Vec<u8>,
    /// The record payload bytes.
    pub payload: Vec<u8>,
}

/// An NFC tag that has been scanned.
#[derive(Deserialize)]
pub struct NfcTag {
    /// The tag identifier, as reported by the operating system.
    pub id: String,
    /// The technology the tag supports.
    pub kind: String,
    /// The NDEF records stored on the tag. Empty when the tag holds no NDEF message.
    pub records: Vec<NfcTagRecord>,
}

/// Response of the [`Nfc::scan`](crate::Nfc::scan) API.
#[derive(Deserialize)]
pub struct ScanResponse {
    /// The tag that has been scanned.
    pub tag: NfcTag,
}

/// Filters the tags to scan by the URI of their NDEF payload.
///
/// Every field is optional and only the ones that are set take part in the filter.
/// **Android only**: the iOS implementation ignores this filter.
#[derive(Debug, Default, Serialize)]
pub struct UriFilter {
    /// Only match URIs with this scheme, e.g. `https`.
    scheme: Option<String>,
    /// Only match URIs with this authority (host), e.g. `tauri.app`.
    host: Option<String>,
    /// Only match URIs whose path starts with this prefix, e.g. `/docs`.
    path_prefix: Option<String>,
}

/// The NFC technologies a tag can support, mirroring the `android.nfc.tech` classes.
///
/// **Android only**. Serialized as the technology name, e.g. `"IsoDep"`.
#[derive(Debug)]
pub enum TechKind {
    /// ISO-DEP (ISO 14443-4) properties and I/O operations.
    IsoDep,
    /// MIFARE Classic properties and I/O operations.
    MifareClassic,
    /// MIFARE Ultralight and MIFARE Ultralight C properties and I/O operations.
    MifareUltralight,
    /// NDEF data and operations on tags that are already formatted as NDEF.
    Ndef,
    /// Formatting operations on tags that can be formatted as NDEF but are not yet.
    NdefFormatable,
    /// NFC-A (ISO 14443-3A) properties and I/O operations.
    NfcA,
    /// NFC-B (ISO 14443-3B) properties and I/O operations.
    NfcB,
    /// NFC Barcode (Kovio NFC Barcode) properties and I/O operations.
    NfcBarcode,
    /// NFC-F (JIS 6319-4) properties and I/O operations.
    NfcF,
    /// NFC-V (ISO 15693) properties and I/O operations.
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

/// The kind of scan to perform, which defines which tags are matched.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanKind {
    /// Only match tags that carry an NDEF message.
    Ndef {
        /// Only match tags whose NDEF payload has this MIME type, e.g. `text/plain`.
        /// **Android only**.
        mime_type: Option<String>,
        /// Only match tags whose NDEF payload URI matches this filter. **Android only**.
        uri: Option<UriFilter>,
        /// Only match tags supporting the listed technologies.
        ///
        /// Each tech list is considered independently and the tag matches when any single tech
        /// list matches it, which provides AND (inside a list) and OR (between lists) semantics.
        ///
        /// **Android only**. See
        /// <https://developer.android.com/reference/android/nfc/NfcAdapter#ACTION_TECH_DISCOVERED>
        /// for more information.
        tech_list: Option<Vec<Vec<TechKind>>>,
    },
    /// Match any tag that is discovered, whether it carries an NDEF message or not.
    Tag {
        /// Only match tags whose payload has this MIME type, e.g. `text/plain`. **Android only**.
        mime_type: Option<String>,
        /// Only match tags whose payload URI matches this filter. **Android only**.
        uri: Option<UriFilter>,
    },
}
