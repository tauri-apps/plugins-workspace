// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Result, Update, UpdaterExt};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Manager, ResourceId, Runtime};

use std::time::Duration;
use url::Url;

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
    rid: Option<ResourceId>,
    available: bool,
    current_version: String,
    version: String,
    date: Option<String>,
    body: Option<String>,
}

#[tauri::command]
pub(crate) async fn check<R: Runtime>(
    app: AppHandle<R>,
    headers: Option<Vec<(String, String)>>,
    timeout: Option<u64>,
    proxy: Option<String>,
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
    if let Some(ref proxy) = proxy {
        let url = Url::parse(proxy.as_str())?;
        builder = builder.proxy(url);
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
        metadata.rid = Some(app.resources_table().add(update));
    }

    Ok(metadata)
}

#[tauri::command]
pub(crate) async fn download_and_install<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
    on_event: Channel,
) -> Result<()> {
    let update = app.resources_table().get::<Update>(rid)?;

    let mut first_chunk = true;

    update
        .download_and_install(
            |chunk_length, content_length| {
                if first_chunk {
                    first_chunk = !first_chunk;
                    let _ = on_event.send(DownloadEvent::Started { content_length });
                }
                let _ = on_event.send(DownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = on_event.send(&DownloadEvent::Finished);
            },
        )
        .await?;

    Ok(())
}
