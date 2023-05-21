// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use authenticator::{
    authenticatorservice::AuthenticatorService, statecallback::StateCallback,
    AuthenticatorTransports, KeyHandle, RegisterFlags, SignFlags, StatusUpdate,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use once_cell::sync::Lazy;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io;
use std::sync::mpsc::channel;
use std::{convert::Into, sync::Mutex};

static MANAGER: Lazy<Mutex<AuthenticatorService>> = Lazy::new(|| {
    let manager = AuthenticatorService::new().expect("The auth service should initialize safely");
    Mutex::new(manager)
});

pub fn init_usb() {
    let mut manager = MANAGER.lock().unwrap();
    // theres also "add_detected_transports()" in the docs?
    manager.add_u2f_usb_hid_platform_transports();
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub key_handle: String,
    pub pubkey: String,
    pub register_data: String,
    pub client_data: String,
}

pub fn register(application: String, timeout: u64, challenge: String) -> crate::Result<String> {
    let (chall_bytes, app_bytes, client_data_string) =
        format_client_data(application.as_str(), challenge.as_str());

    // log the status rx?
    let (status_tx, _status_rx) = channel::<StatusUpdate>();

    let mut manager = MANAGER.lock().unwrap();

    let (register_tx, register_rx) = channel();
    let callback = StateCallback::new(Box::new(move |rv| {
        register_tx.send(rv).unwrap();
    }));

    let res = manager.register(
        RegisterFlags::empty(),
        timeout,
        chall_bytes,
        app_bytes,
        vec![],
        status_tx,
        callback,
    );

    match res {
        Ok(_r) => {
            let register_result = register_rx
                .recv()
                .expect("Problem receiving, unable to continue");

            if let Err(e) = register_result {
                return Err(e.into());
            }

            let (register_data, device_info) = register_result.unwrap(); // error already has been checked

            // println!("Register result: {}", base64::encode(&register_data));
            println!("Device info: {}", &device_info);

            let (key_handle, public_key) =
                _u2f_get_key_handle_and_public_key_from_register_response(&register_data).unwrap();
            let key_handle_base64 = URL_SAFE_NO_PAD.encode(key_handle);
            let public_key_base64 = URL_SAFE_NO_PAD.encode(public_key);
            let register_data_base64 = URL_SAFE_NO_PAD.encode(&register_data);
            println!("Key Handle: {}", &key_handle_base64);
            println!("Public Key: {}", &public_key_base64);

            // Ok(base64::encode(&register_data))
            // Ok(key_handle_base64)
            let res = serde_json::to_string(&Registration {
                key_handle: key_handle_base64,
                pubkey: public_key_base64,
                register_data: register_data_base64,
                client_data: client_data_string,
            })?;
            Ok(res)
        }
        Err(e) => Err(e.into()),
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub key_handle: String,
    pub sign_data: String,
}

pub fn sign(
    application: String,
    timeout: u64,
    challenge: String,
    key_handle: String,
) -> crate::Result<String> {
    let credential = match URL_SAFE_NO_PAD.decode(key_handle) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.into());
        }
    };
    let key_handle = KeyHandle {
        credential,
        transports: AuthenticatorTransports::empty(),
    };

    let (chall_bytes, app_bytes, _) = format_client_data(application.as_str(), challenge.as_str());

    let (sign_tx, sign_rx) = channel();
    let callback = StateCallback::new(Box::new(move |rv| {
        sign_tx.send(rv).unwrap();
    }));

    // log the status rx?
    let (status_tx, _status_rx) = channel::<StatusUpdate>();

    let mut manager = MANAGER.lock().unwrap();

    let res = manager.sign(
        SignFlags::empty(),
        timeout,
        chall_bytes,
        vec![app_bytes],
        vec![key_handle],
        status_tx,
        callback,
    );
    match res {
        Ok(_v) => {
            let sign_result = sign_rx
                .recv()
                .expect("Problem receiving, unable to continue");

            if let Err(e) = sign_result {
                return Err(e.into());
            }

            let (_, handle_used, sign_data, device_info) = sign_result.unwrap();

            let sig = URL_SAFE_NO_PAD.encode(sign_data);

            println!("Sign result: {sig}");
            println!("Key handle used: {}", URL_SAFE_NO_PAD.encode(&handle_used));
            println!("Device info: {}", &device_info);
            println!("Done.");

            let res = serde_json::to_string(&Signature {
                sign_data: sig,
                key_handle: URL_SAFE_NO_PAD.encode(&handle_used),
            })?;
            Ok(res)
        }
        Err(e) => Err(e.into()),
    }
}

fn format_client_data(application: &str, challenge: &str) -> (Vec<u8>, Vec<u8>, String) {
    let d =
        format!(r#"{{"challenge": "{challenge}", "version": "U2F_V2", "appId": "{application}"}}"#);
    let mut challenge = Sha256::new();
    challenge.update(d.as_bytes());
    let chall_bytes = challenge.finalize().to_vec();

    let mut app = Sha256::new();
    app.update(application.as_bytes());
    let app_bytes = app.finalize().to_vec();

    (chall_bytes, app_bytes, d)
}

fn _u2f_get_key_handle_and_public_key_from_register_response(
    register_response: &[u8],
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    if register_response[0] != 0x05 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Reserved byte not set correctly",
        ));
    }

    // 1: reserved
    // 65: public key
    // 1: key handle length
    // key handle
    // x.509 cert
    // sig

    let key_handle_len = register_response[66] as usize;
    let mut public_key = register_response.to_owned();
    let mut key_handle = public_key.split_off(67);
    let _attestation = key_handle.split_off(key_handle_len);

    // remove fist (reserved) and last (handle len) bytes
    let pk: Vec<u8> = public_key[1..public_key.len() - 1].to_vec();

    Ok((key_handle, pk))
}
