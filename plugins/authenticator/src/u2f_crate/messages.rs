// Copyright 2021 Flavio Oliveira
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// As defined by FIDO U2F Javascript API.
// https://fidoalliance.org/specs/fido-u2f-v1.0-nfc-bt-amendment-20150514/fido-u2f-javascript-api.html#registration

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct U2fRegisterRequest {
    pub app_id: String,
    pub register_requests: Vec<RegisterRequest>,
    pub registered_keys: Vec<RegisteredKey>,
}

#[derive(Serialize)]
pub struct RegisterRequest {
    pub version: String,
    pub challenge: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredKey {
    pub version: String,
    pub key_handle: Option<String>,
    pub app_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub registration_data: String,
    pub version: String,
    pub client_data: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct U2fSignRequest {
    pub app_id: String,
    pub challenge: String,
    pub registered_keys: Vec<RegisteredKey>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignResponse {
    pub key_handle: String,
    pub signature_data: String,
    pub client_data: String,
}
