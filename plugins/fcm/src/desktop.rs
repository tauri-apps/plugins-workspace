use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Fcm<R>> {
  Ok(Fcm(app.clone()))
}

/// Access to the fcm APIs.
pub struct Fcm<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Fcm<R> {
  pub fn get_latest_notification_data(&self, _payload: GetLatestNotificationDataRequest) -> crate::Result<GetLatestNotificationDataResponse> {
    Err(crate::Error::Other("Not implemented on desktop".to_string()))
  }

  pub fn get_token(&self, _payload: GetTokenRequest) -> crate::Result<GetTokenResponse> {
    Err(crate::Error::Other("Not implemented on desktop".to_string()))  
  }

  pub fn subscribe_to_topic(&self, _payload: SubscribeToTopicRequest) -> crate::Result<SubscribeToTopicResponse> {
    Err(crate::Error::Other("Not implemented on desktop".to_string()))
  }
}