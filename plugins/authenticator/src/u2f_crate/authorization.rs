// Copyright 2021 Flavio Oliveira
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use bytes::{Buf, BufMut};
use openssl::sha::sha256;
use serde::Serialize;
use std::io::Cursor;

use crate::u2f_crate::u2ferror::U2fError;

/// The `Result` type used in this crate.
type Result<T> = ::std::result::Result<T, U2fError>;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Authorization {
    pub counter: u32,
    pub user_presence: bool,
}

pub fn parse_sign_response(
    app_id: String,
    client_data: Vec<u8>,
    public_key: Vec<u8>,
    sign_data: Vec<u8>,
) -> Result<Authorization> {
    if sign_data.len() <= 5 {
        return Err(U2fError::InvalidSignatureData);
    }

    let user_presence_flag = &sign_data[0];
    let counter = &sign_data[1..=4];
    let signature = &sign_data[5..];

    // Let's build the msg to verify the signature
    let app_id_hash = sha256(&app_id.into_bytes());
    let client_data_hash = sha256(&client_data[..]);

    let mut msg = vec![];
    msg.put(app_id_hash.as_ref());
    msg.put_u8(*user_presence_flag);
    msg.put(counter);
    msg.put(client_data_hash.as_ref());

    let public_key = super::crypto::NISTP256Key::from_bytes(&public_key)?;

    // The signature is to be verified by the relying party using the public key obtained during registration.
    let verified = public_key.verify_signature(signature, msg.as_ref())?;
    if !verified {
        return Err(U2fError::BadSignature);
    }

    let authorization = Authorization {
        counter: get_counter(counter),
        user_presence: true,
    };

    Ok(authorization)
}

fn get_counter(counter: &[u8]) -> u32 {
    let mut buf = Cursor::new(counter);
    buf.get_u32()
}
