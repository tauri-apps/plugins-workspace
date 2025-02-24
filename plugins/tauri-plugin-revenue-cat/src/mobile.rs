use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_revenue_cat);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<RevenueCat<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.revenue-cat", "ExamplePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_revenue_cat)?;
  Ok(RevenueCat(handle))
}

/// Access to the revenue-cat APIs.
pub struct RevenueCat<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> RevenueCat<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }
}
