// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{PendingUpdate, Result, UpdaterExt};

use http::header;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::{api::ipc::Channel, AppHandle, Runtime, State};

use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DownloadProgress {
    chunk_length: usize,
    content_length: Option<u64>,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    available: bool,
    current_version: String,
    version: String,
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
    pending: State<'_, PendingUpdate>,
    headers: Option<HeaderMap>,
    timeout: Option<u64>,
    target: Option<String>,
) -> Result<Metadata> {
    let mut builder = app.updater_builder();
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

    let updater = builder.build()?;
    let update = updater.check().await?;
    let mut metadata = Metadata::default();
    if let Some(update) = update {
        metadata.available = true;
        metadata.current_version = update.current_version.clone();
        metadata.version = update.version.clone();
        metadata.date = update.date.map(|d| d.to_string());
        metadata.body = update.body.clone();
        pending.0.lock().await.replace(update);
    }

    Ok(metadata)
}

#[tauri::command]
pub(crate) async fn download_and_install<R: Runtime>(
    _app: AppHandle<R>,
    pending: State<'_, PendingUpdate>,
    on_event: Channel<R>,
) -> Result<()> {
    if let Some(pending) = &*pending.0.lock().await {
        let first_chunk = AtomicBool::new(false);
        let on_event_c = on_event.clone();
        pending
            .download_and_install(
                move |chunk_length, content_length| {
                    if first_chunk.swap(false, Ordering::Acquire) {
                        on_event
                            .send(&DownloadEvent::Started { content_length })
                            .unwrap();
                    }
                    on_event
                        .send(&DownloadEvent::Progress { chunk_length })
                        .unwrap();
                },
                move || {
                    on_event_c.send(&DownloadEvent::Finished).unwrap();
                },
            )
            .await?;
    }
    Ok(())
}
