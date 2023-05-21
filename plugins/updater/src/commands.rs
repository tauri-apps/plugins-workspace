// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{PendingUpdate, Result, UpdaterExt};

use http::header;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::{api::ipc::Channel, AppHandle, Runtime, State};

use std::{collections::HashMap, time::Duration};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    available: bool,
    current_version: String,
    latest_version: String,
    date: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Default)]
pub(crate) struct HeaderMap(header::HeaderMap);

impl<'de> Deserialize<'de> for HeaderMap {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, String>::deserialize(deserializer)?;
        let mut headers = header::HeaderMap::default();
        for (key, value) in map {
            if let (Ok(key), Ok(value)) = (
                header::HeaderName::from_bytes(key.as_bytes()),
                header::HeaderValue::from_str(&value),
            ) {
                headers.insert(key, value);
            } else {
                return Err(serde::de::Error::custom(format!(
                    "invalid header `{key}` `{value}`"
                )));
            }
        }
        Ok(Self(headers))
    }
}

#[tauri::command]
pub(crate) async fn check<R: Runtime>(
    app: AppHandle<R>,
    pending: State<'_, PendingUpdate<R>>,
    headers: Option<HeaderMap>,
    timeout: Option<u64>,
    target: Option<String>,
) -> Result<Metadata> {
    let mut builder = app.updater();
    if let Some(headers) = headers {
        for (k, v) in headers.0.iter() {
            builder = builder.header(k, v)?;
        }
    }
    if let Some(timeout) = timeout {
        builder = builder.timeout(Duration::from_secs(timeout));
    }
    if let Some(target) = target {
        builder = builder.target(target);
    }

    let response = builder.check().await?;

    let metadata = Metadata {
        available: response.is_update_available(),
        current_version: response.current_version().to_string(),
        latest_version: response.latest_version().to_string(),
        date: response.date().map(|d| d.to_string()),
        body: response.body().cloned(),
    };

    pending.0.lock().await.replace(response);

    Ok(metadata)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadProgress {
    chunk_length: usize,
    content_length: Option<u64>,
}

#[tauri::command]
pub(crate) async fn download_and_install<R: Runtime>(
    _app: AppHandle<R>,
    pending: State<'_, PendingUpdate<R>>,
    on_event: Channel<R>,
) -> Result<()> {
    if let Some(pending) = &*pending.0.lock().await {
        pending
            .download_and_install(move |event| {
                on_event.send(&event).unwrap();
            })
            .await?;
    }
    Ok(())
}
