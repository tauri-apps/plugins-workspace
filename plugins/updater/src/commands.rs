// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{PendingUpdate, Result, UpdaterExt};

use serde::Serialize;
use tauri::{api::ipc::Channel, AppHandle, Runtime, State};

use std::{
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

#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Metadata {
    available: bool,
    current_version: String,
    version: String,
    date: Option<String>,
    body: Option<String>,
}

#[tauri::command]
pub(crate) async fn check<R: Runtime>(
    app: AppHandle<R>,
    pending: State<'_, PendingUpdate>,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<u64>,
    target: Option<String>,
) -> Result<Metadata> {
    let mut builder = app.updater_builder();
    if let Some(headers) = headers {
        for (k, v) in headers {
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
