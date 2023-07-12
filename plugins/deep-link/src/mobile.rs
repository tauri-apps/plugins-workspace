// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{
    ipc::{Channel, InvokeBody},
    plugin::{PluginApi, PluginHandle},
    AppHandle, Manager, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.deep_link";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_deep_link);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<DeepLink<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "DeepLinkPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_deep_link)?;

    #[cfg(target_os = "android")]
    let app_handle = _app.clone();
    #[cfg(target_os = "android")]
    handle
        .run_mobile_plugin::<()>(
            "setEventHandler",
            EventHandler {
                handler: Channel::new(move |event| {
                    println!("got channel event: {:?}", &event);

                    let url = match event {
                        InvokeBody::Json(payload) => payload
                            .get("url")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_owned()),
                        _ => None,
                    };

                    app_handle.trigger_global("deep-link://new-url", url.clone());
                    app_handle.emit_all("deep-link://new-url", url).unwrap(); // TODO: Replace unwrap with let _ binding
                    Ok(())
                }),
            },
        )
        .unwrap(); // TODO: Don't unwrap here.

    Ok(DeepLink(handle))
}

/// Access to the deep-link APIs.
pub struct DeepLink<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> DeepLink<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        self.0
            .run_mobile_plugin("ping", payload)
            .map_err(Into::into)
    }

    // TODO: URI instead of String?
    /// Get the last saved URL that triggered the deep link.
    pub fn get_last_link(&self) -> crate::Result<Option<String>> {
        self.0
            .run_mobile_plugin::<LastUrl>("getLastLink", ())
            .map(|v| v.url)
            .map_err(Into::into)
    }
}
