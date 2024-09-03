use serde::Deserialize;


#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[cfg(target_os = "android")]
    pub android_app_id: String,

    #[cfg(target_os = "android")]
    pub project_id: String,

    #[cfg(target_os = "ios")]
    pub ios_app_id: String,

    pub api_key: String,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {}