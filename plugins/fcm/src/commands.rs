use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::FcmExt;
use crate::Result;

#[command]
pub(crate) async fn get_latest_notification_data<R: Runtime>(
    app: AppHandle<R>,
    payload: GetLatestNotificationDataRequest,
) -> Result<GetLatestNotificationDataResponse> {
    app.fcm().get_latest_notification_data(payload)
}

#[command]
pub(crate) async fn get_token<R: Runtime>(
    app: AppHandle<R>,
    payload: GetTokenRequest,
) -> Result<GetTokenResponse> {
    app.fcm().get_token(payload)
}

#[command]
pub(crate) async fn subscribe_to_topic<R: Runtime>(
    app: AppHandle<R>,
    payload: SubscribeToTopicRequest,
) -> Result<SubscribeToTopicResponse> {
    app.fcm().subscribe_to_topic(payload)
}

// #[command]
// pub(crate) async fn unsubscribe_from_topic<R: Runtime>(
//     app: AppHandle<R>,
//     payload: SubscribeToTopicRequest,
// ) -> Result<SubscribeToTopicResponse> {
//     app.fcm().unsubscribe_from_topic(payload)
// }
