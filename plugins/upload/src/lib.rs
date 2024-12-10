// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Upload files from disk to a remote server over HTTP.
//!
//! Download files from a remote HTTP server to disk.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

mod transfer_stats;
use transfer_stats::TransferStats;

use futures_util::TryStreamExt;
use serde::{ser::Serializer, Serialize};
use tauri::{
    command,
    ipc::Channel,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Runtime,
};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::codec::{BytesCodec, FramedRead};

use read_progress_stream::ReadProgressStream;

use std::collections::HashMap;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("{0}")]
    ContentLength(String),
    #[error("request failed with status code {0}: {1}")]
    HttpErrorCode(u16, String),
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
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    progress: u64,
    progress_total: u64,
    total: u64,
    transfer_speed: u64,
}

#[command]
async fn download(
    url: &str,
    file_path: &str,
    headers: HashMap<String, String>,
    body: Option<String>,
    on_progress: Channel<ProgressPayload>,
) -> Result<()> {
    let client = reqwest::Client::new();
    let mut request = if let Some(body) = body {
        client.post(url).body(body)
    } else {
        client.get(url)
    };
    // Loop trought the headers keys and values
    // and add them to the request object.
    for (key, value) in headers {
        request = request.header(&key, value);
    }

    let response = request.send().await?;
    if !response.status().is_success() {
        return Err(Error::HttpErrorCode(
            response.status().as_u16(),
            response.text().await.unwrap_or_default(),
        ));
    }
    let total = response.content_length().unwrap_or(0);

    let mut file = BufWriter::new(File::create(file_path).await?);
    let mut stream = response.bytes_stream();

    let mut stats = TransferStats::default();
    while let Some(chunk) = stream.try_next().await? {
        file.write_all(&chunk).await?;
        stats.record_chunk_transfer(chunk.len());
        let _ = on_progress.send(ProgressPayload {
            progress: chunk.len() as u64,
            progress_total: stats.total_transferred,
            total,
            transfer_speed: stats.transfer_speed,
        });
    }
    file.flush().await?;

    Ok(())
}

#[command]
async fn upload(
    url: &str,
    file_path: &str,
    headers: HashMap<String, String>,
    on_progress: Channel<ProgressPayload>,
) -> Result<String> {
    // Read the file
    let file = File::open(file_path).await?;
    let file_len = file.metadata().await.unwrap().len();

    // Create the request and attach the file to the body
    let client = reqwest::Client::new();
    let mut request = client
        .post(url)
        .header(reqwest::header::CONTENT_LENGTH, file_len)
        .body(file_to_body(on_progress, file));

    // Loop through the headers keys and values
    // and add them to the request object.
    for (key, value) in headers {
        request = request.header(&key, value);
    }

    let response = request.send().await?;
    if response.status().is_success() {
        response.text().await.map_err(Into::into)
    } else {
        Err(Error::HttpErrorCode(
            response.status().as_u16(),
            response.text().await.unwrap_or_default(),
        ))
    }
}

fn file_to_body(channel: Channel<ProgressPayload>, file: File) -> reqwest::Body {
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|r| r.freeze());

    let mut stats = TransferStats::default();
    reqwest::Body::wrap_stream(ReadProgressStream::new(
        stream,
        Box::new(move |progress, total| {
            stats.record_chunk_transfer(progress as usize);
            let _ = channel.send(ProgressPayload {
                progress,
                progress_total: stats.total_transferred,
                total,
                transfer_speed: stats.transfer_speed,
            });
        }),
    ))
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("upload")
        .invoke_handler(tauri::generate_handler![download, upload])
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{self, Mock, Server, ServerGuard};
    use tauri::ipc::InvokeResponseBody;
    struct MockedServer {
        _server: ServerGuard,
        url: String,
        mocked_endpoint: Mock,
    }

    #[tokio::test]
    async fn should_error_if_status_not_success() {
        let mocked_server = spawn_server_mocked(400).await;
        let result = download_file(&mocked_server.url).await;
        mocked_server.mocked_endpoint.assert();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_download_file_successfully() {
        let mocked_server = spawn_server_mocked(200).await;
        let result = download_file(&mocked_server.url).await;
        mocked_server.mocked_endpoint.assert();
        assert!(
            result.is_ok(),
            "failed to download file: {}",
            result.unwrap_err()
        );
    }

    async fn download_file(url: &str) -> Result<()> {
        let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test/test.txt");
        let headers = HashMap::new();
        let sender: Channel<ProgressPayload> =
            Channel::new(|msg: InvokeResponseBody| -> tauri::Result<()> {
                let _ = msg;
                Ok(())
            });
        download(url, file_path, headers, None, sender).await
    }

    async fn spawn_server_mocked(return_status: usize) -> MockedServer {
        let mut _server = Server::new_async().await;
        let path = "/mock_test";
        let mock = _server
            .mock("GET", path)
            .with_status(return_status)
            .with_body("mocked response body")
            .create_async()
            .await;

        let url = _server.url() + path;
        MockedServer {
            _server,
            url,
            mocked_endpoint: mock,
        }
    }
}
