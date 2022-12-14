// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod auth;
mod error;
mod u2f;

use tauri::{plugin::Plugin, Invoke, Runtime};

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

#[tauri::command]
fn init() {
    auth::init_usb();
}

#[tauri::command]
fn register(timeout: u64, challenge: String, application: String) -> crate::Result<String> {
    auth::register(application, timeout, challenge)
}

#[tauri::command]
fn verify_registration(
    challenge: String,
    application: String,
    register_data: String,
    client_data: String,
) -> crate::Result<String> {
    u2f::verify_registration(application, challenge, register_data, client_data)
}

#[tauri::command]
fn sign(
    timeout: u64,
    challenge: String,
    application: String,
    key_handle: String,
) -> crate::Result<String> {
    auth::sign(application, timeout, challenge, key_handle)
}

#[tauri::command]
fn verify_signature(
    challenge: String,
    application: String,
    sign_data: String,
    client_data: String,
    key_handle: String,
    pubkey: String,
) -> crate::Result<u32> {
    u2f::verify_signature(
        application,
        challenge,
        sign_data,
        client_data,
        key_handle,
        pubkey,
    )
}

pub struct TauriAuthenticator<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for TauriAuthenticator<R> {
    fn default() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![
                init,
                register,
                verify_registration,
                sign,
                verify_signature
            ]),
        }
    }
}

impl<R: Runtime> Plugin<R> for TauriAuthenticator<R> {
    fn name(&self) -> &'static str {
        "authenticator"
    }

    fn extend_api(&mut self, invoke: Invoke<R>) {
        (self.invoke_handler)(invoke)
    }
}
