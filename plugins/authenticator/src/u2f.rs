// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::u2f_crate::messages::*;
use crate::u2f_crate::protocol::*;
use crate::u2f_crate::register::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::prelude::*;
use serde::Serialize;
use std::convert::Into;

static VERSION: &str = "U2F_V2";

pub fn make_challenge(app_id: &str, challenge_bytes: Vec<u8>) -> Challenge {
    let utc: DateTime<Utc> = Utc::now();
    Challenge {
        challenge: URL_SAFE_NO_PAD.encode(challenge_bytes),
        timestamp: format!("{utc:?}"),
        app_id: app_id.to_string(),
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationVerification {
    pub key_handle: String,
    pub pubkey: String,
    pub device_name: Option<String>,
}

pub fn verify_registration(
    app_id: String,
    challenge: String,
    register_data: String,
    client_data: String,
) -> crate::Result<String> {
    let challenge_bytes = URL_SAFE_NO_PAD.decode(challenge)?;
    let challenge = make_challenge(&app_id, challenge_bytes);
    let client_data_bytes: Vec<u8> = client_data.as_bytes().into();
    let client_data_base64 = URL_SAFE_NO_PAD.encode(client_data_bytes);
    let client = U2f::new(app_id);
    match client.register_response(
        challenge,
        RegisterResponse {
            registration_data: register_data,
            client_data: client_data_base64,
            version: VERSION.to_string(),
        },
    ) {
        Ok(v) => {
            let rv = RegistrationVerification {
                key_handle: URL_SAFE_NO_PAD.encode(&v.key_handle),
                pubkey: URL_SAFE_NO_PAD.encode(&v.pub_key),
                device_name: v.device_name,
            };
            Ok(serde_json::to_string(&rv)?)
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignatureVerification {
    pub counter: u8,
}

pub fn verify_signature(
    app_id: String,
    challenge: String,
    sign_data: String,
    client_data: String,
    key_handle: String,
    pub_key: String,
) -> crate::Result<u32> {
    let challenge_bytes = URL_SAFE_NO_PAD.decode(challenge)?;
    let chal = make_challenge(&app_id, challenge_bytes);
    let client_data_bytes: Vec<u8> = client_data.as_bytes().into();
    let client_data_base64 = URL_SAFE_NO_PAD.encode(client_data_bytes);
    let key_handle_bytes = URL_SAFE_NO_PAD.decode(&key_handle)?;
    let pubkey_bytes = URL_SAFE_NO_PAD.decode(pub_key)?;
    let client = U2f::new(app_id);
    let mut _counter: u32 = 0;
    match client.sign_response(
        chal,
        Registration {
            // here only needs pubkey and keyhandle
            key_handle: key_handle_bytes,
            pub_key: pubkey_bytes,
            attestation_cert: None,
            device_name: None,
        },
        SignResponse {
            // here needs client data and sig data and key_handle
            signature_data: sign_data,
            client_data: client_data_base64,
            key_handle,
        },
        _counter,
    ) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.into()),
    }
}
