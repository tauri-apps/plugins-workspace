use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde::{ser::Serializer, Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri::{
    api::ipc::{format_callback, CallbackFn},
    plugin::Plugin,
    AppHandle, Invoke, Manager, Runtime, State, Window,
};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{
        protocol::{CloseFrame as ProtocolCloseFrame, WebSocketConfig},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};

use std::collections::HashMap;

type Id = u32;
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketWriter = SplitSink<WebSocket, Message>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("connection not found for the given id: {0}")]
    ConnectionNotFound(Id),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Default)]
struct ConnectionManager(Mutex<HashMap<Id, WebSocketWriter>>);

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub max_send_queue: Option<usize>,
    pub max_message_size: Option<usize>,
    pub max_frame_size: Option<usize>,
    pub accept_unmasked_frames: bool,
}

impl From<ConnectionConfig> for WebSocketConfig {
    fn from(config: ConnectionConfig) -> Self {
        Self {
            max_send_queue: config.max_send_queue,
            max_message_size: config.max_message_size,
            max_frame_size: config.max_frame_size,
            accept_unmasked_frames: config.accept_unmasked_frames,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}

#[tauri::command]
fn connect<R: Runtime>(
    window: Window<R>,
    url: String,
    callback_function: CallbackFn,
    config: Option<ConnectionConfig>,
) -> Result<Id> {
    let id = rand::random();
    let (ws_stream, _) =
        tauri::async_runtime::block_on(connect_async_with_config(url, config.map(Into::into)))?;

    tauri::async_runtime::spawn(async move {
        let (write, read) = ws_stream.split();
        let manager = window.state::<ConnectionManager>();
        manager.0.lock().await.insert(id, write);
        read.for_each(move |message| {
            let window_ = window.clone();
            async move {
                if let Ok(Message::Close(_)) = message {
                    let manager = window_.state::<ConnectionManager>();
                    manager.0.lock().await.remove(&id);
                }

                let response = match message {
                    Ok(Message::Text(t)) => {
                        serde_json::to_value(WebSocketMessage::Text(t)).unwrap()
                    }
                    Ok(Message::Binary(t)) => {
                        serde_json::to_value(WebSocketMessage::Binary(t)).unwrap()
                    }
                    Ok(Message::Ping(t)) => {
                        serde_json::to_value(WebSocketMessage::Ping(t)).unwrap()
                    }
                    Ok(Message::Pong(t)) => {
                        serde_json::to_value(WebSocketMessage::Pong(t)).unwrap()
                    }
                    Ok(Message::Close(t)) => {
                        serde_json::to_value(WebSocketMessage::Close(t.map(|v| CloseFrame {
                            code: v.code.into(),
                            reason: v.reason.into_owned(),
                        })))
                        .unwrap()
                    }
                    Ok(Message::Frame(_)) => serde_json::Value::Null, // This value can't be recieved.
                    Err(e) => serde_json::to_value(Error::from(e)).unwrap(),
                };
                let js = format_callback(callback_function, &response)
                    .expect("unable to serialize websocket message");
                let _ = window_.eval(js.as_str());
            }
        })
        .await;
    });

    Ok(id)
}

#[tauri::command]
async fn send(
    manager: State<'_, ConnectionManager>,
    id: Id,
    message: WebSocketMessage,
) -> Result<()> {
    if let Some(write) = manager.0.lock().await.get_mut(&id) {
        write
            .send(match message {
                WebSocketMessage::Text(t) => Message::Text(t),
                WebSocketMessage::Binary(t) => Message::Binary(t),
                WebSocketMessage::Ping(t) => Message::Ping(t),
                WebSocketMessage::Pong(t) => Message::Pong(t),
                WebSocketMessage::Close(t) => Message::Close(t.map(|v| ProtocolCloseFrame {
                    code: v.code.into(),
                    reason: std::borrow::Cow::Owned(v.reason),
                })),
            })
            .await?;
        Ok(())
    } else {
        Err(Error::ConnectionNotFound(id))
    }
}

pub struct TauriWebsocket<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for TauriWebsocket<R> {
    fn default() -> Self {
        Self {
            invoke_handler: Box::new(tauri::generate_handler![connect, send]),
        }
    }
}

impl<R: Runtime> Plugin<R> for TauriWebsocket<R> {
    fn name(&self) -> &'static str {
        "websocket"
    }

    fn initialize(&mut self, app: &AppHandle<R>, _config: JsonValue) -> tauri::plugin::Result<()> {
        app.manage(ConnectionManager::default());
        Ok(())
    }

    fn extend_api(&mut self, invoke: Invoke<R>) {
        (self.invoke_handler)(invoke)
    }
}
