// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use futures::TryStreamExt;
use serde::{ser::Serializer, Serialize};
use tauri::{command, plugin::Plugin, Invoke, Runtime, Window};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use read_progress_stream::ReadProgressStream;

use std::{collections::HashMap, sync::Mutex};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Clone, Serialize)]
struct ProgressPayload {
    id: u32,
    progress: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
struct FileSizePayload {
    id: u32,
    size: u64,
}

#[derive(Clone, Serialize)]
struct ResponseData {
    text: String,
    status: u16,
}

#[command]
async fn upload<R: Runtime>(
    window: Window<R>,
    id: u32,
    url: &str,
    file_path: &str,
    headers: HashMap<String, String>,
) -> Result<ResponseData> {
    // Read the file
    let parsed_url = url::Url::parse(url).unwrap();
    let file = File::open(file_path).await?;
    let file_metadata = file.metadata().await?;
    let file_size = file_metadata.len();
    let _ = window.emit(
        "upload://file-size",
        FileSizePayload {
            size: file_size,
            id,
        },
    );

    // Create the request and attach the file to the body
    let client = reqwest::Client::new();
    let mut request = client.put(url).body(file_to_body(id, window, file));

    request = request.header("Content-Length", file_size);
    request = request.header("User-Agent", "Tauri/1.0");
    request = request.header("Accept-Encoding", "gzip, deflate, br");
    request = request.header("Accept", "*/*");
    request = request.header("Connection", "keep-alive");
    let host = parsed_url.host().unwrap();
    request = request.header("Host", host.to_string());

    // Loop trough the headers keys and values
    // and add them to the request object.
    for (key, value) in headers {
        request = request.header(&key, value);
    }

    let response = request.send().await?;

    let status = response.status().as_u16();
    let text = response.text().await?;
    Ok(ResponseData { text, status })
}

fn file_to_body<R: Runtime>(id: u32, window: Window<R>, file: File) -> reqwest::Body {
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|r| r.freeze());
    let window = Mutex::new(window);
    reqwest::Body::wrap_stream(ReadProgressStream::new(
        stream,
        Box::new(move |progress, total| {
            let _ = window.lock().unwrap().emit(
                "upload://progress",
                ProgressPayload {
                    id,
                    progress,
                    total,
                },
            );
        }),
    ))
}

/// Tauri plugin.
pub struct Upload<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for Upload<R> {
    fn default() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![upload]),
        }
    }
}

impl<R: Runtime> Plugin<R> for Upload<R> {
    fn name(&self) -> &'static str {
        "upload"
    }

    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }
}
