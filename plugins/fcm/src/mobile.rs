use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.fcm";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_fcm);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Fcm<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "FCMPlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_fcm)?;
  Ok(Fcm(handle))
}

/// Access to the fcm APIs.
pub struct Fcm<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Fcm<R> {
  pub fn get_latest_notification_data(&self, payload: GetLatestNotificationDataRequest) -> crate::Result<GetLatestNotificationDataResponse> {
    self
      .0
      .run_mobile_plugin("getLatestNotificationData", payload)
      .map_err(Into::into)
  }

  pub fn get_token(&self, payload: GetTokenRequest) -> crate::Result<GetTokenResponse> {
    self
      .0
      .run_mobile_plugin("getToken", payload)
      .map_err(Into::into)
  }

  pub fn subscribe_to_topic(&self, payload: SubscribeToTopicRequest) -> crate::Result<SubscribeToTopicResponse> {
    self
      .0
      .run_mobile_plugin("subscribeToTopic", payload)
      .map_err(Into::into)
  }
}