use base64::prelude::{Engine, BASE64_STANDARD};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use http::{
    header::{HeaderName, HeaderValue},
    Request,
};
use hyper::client::conn;
use hyper_util::rt::TokioIo;
use serde::{ser::Serializer, Deserialize, Serialize};
use tauri::{
    api::ipc::{format_callback, CallbackFn},
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Manager, Runtime, State, Window,
};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    client_async_tls_with_config, connect_async_with_config,
    tungstenite::{
        client::IntoClientRequest,
        error::UrlError,
        protocol::{CloseFrame as ProtocolCloseFrame, WebSocketConfig},
        Message,
    },
    Connector, MaybeTlsStream, WebSocketStream,
};

use std::collections::HashMap;
use std::str::FromStr;

type Id = u32;
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketWriter =
    SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>, Message>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Websocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("connection not found for the given id: {0}")]
    ConnectionNotFound(Id),
    #[error(transparent)]
    InvalidHeaderValue(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue),
    #[error(transparent)]
    InvalidHeaderName(#[from] tokio_tungstenite::tungstenite::http::header::InvalidHeaderName),
    #[error(transparent)]
    ProxyConnectionError(#[from] hyper::Error),
    #[error("proxy returned status code: {0}")]
    ProxyStatusError(u16),
    #[error(transparent)]
    ProxyIoError(std::io::Error),
    #[error(transparent)]
    ProxyHttpError(http::Error),
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

struct TlsConnector(Mutex<Option<Connector>>);
struct ProxyConfigurationInternal(Mutex<Option<ProxyConfiguration>>);

#[derive(Clone)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

impl ProxyAuth {
    pub fn encode(&self) -> String {
        BASE64_STANDARD.encode(format!("{}:{}", self.username, self.password))
    }
}

#[derive(Clone)]
pub struct ProxyConfiguration {
    pub proxy_url: String,
    pub proxy_port: u16,
    pub auth: Option<ProxyAuth>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub write_buffer_size: Option<usize>,
    pub max_write_buffer_size: Option<usize>,
    pub max_message_size: Option<usize>,
    pub max_frame_size: Option<usize>,
    #[serde(default)]
    pub accept_unmasked_frames: bool,
    pub headers: Option<Vec<(String, String)>>,
}

impl From<ConnectionConfig> for WebSocketConfig {
    fn from(config: ConnectionConfig) -> Self {
        // Disabling the warning on max_send_queue which we don't use anymore since it was deprecated.
        #[allow(deprecated)]
        Self {
            max_send_queue: None,
            write_buffer_size: config.write_buffer_size.unwrap_or(128 * 1024),
            max_write_buffer_size: config.max_write_buffer_size.unwrap_or(usize::MAX),
            // This may be harmful since if it's not provided from js we're overwriting the default value with None, meaning no size limit.
            max_message_size: config.max_message_size,
            // This may be harmful since if it's not provided from js we're overwriting the default value with None, meaning no size limit.
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
async fn connect<R: Runtime>(
    window: Window<R>,
    url: String,
    callback_function: CallbackFn,
    config: Option<ConnectionConfig>,
) -> Result<Id> {
    let id = rand::random();
    let mut request = url.into_client_request()?;

    if let Some(headers) = config.as_ref().and_then(|c| c.headers.as_ref()) {
        for (k, v) in headers {
            let header_name = HeaderName::from_str(k.as_str())?;
            let header_value = HeaderValue::from_str(v.as_str())?;
            request.headers_mut().insert(header_name, header_value);
        }
    }

    #[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
    let tls_connector = match window.try_state::<TlsConnector>() {
        Some(tls_connector) => tls_connector.0.lock().await.clone(),
        None => None,
    };
    #[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
    let tls_connector = None;

    let proxy_config = match window.try_state::<ProxyConfigurationInternal>() {
        Some(proxy_config) => proxy_config.0.lock().await.clone(),
        None => None,
    };

    let ws_stream = if let Some(proxy_config) = proxy_config {
        connect_using_proxy(request, config, proxy_config, tls_connector).await?
    } else {
        #[cfg(any(feature = "rustls-tls", feature = "native-tls"))]
        let (ws_stream, _) =
            connect_async_tls_with_config(request, config.map(Into::into), false, tls_connector)
                .await?;
        #[cfg(not(any(feature = "rustls-tls", feature = "native-tls")))]
        let (ws_stream, _) =
            connect_async_with_config(request, config.map(Into::into), false).await?;

        ws_stream
    };

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

async fn connect_using_proxy(
    request: Request<()>,
    config: Option<ConnectionConfig>,
    proxy_config: ProxyConfiguration,
    tls_connector: Option<Connector>,
) -> Result<WebSocket> {
    let domain = domain(&request)?;
    let port = request
        .uri()
        .port_u16()
        .or_else(|| match request.uri().scheme_str() {
            Some("wss") => Some(443),
            Some("ws") => Some(80),
            _ => None,
        })
        .ok_or(Error::Websocket(
            tokio_tungstenite::tungstenite::Error::Url(UrlError::UnsupportedUrlScheme),
        ))?;

    let tcp = TcpStream::connect(format!(
        "{}:{}",
        proxy_config.proxy_url, proxy_config.proxy_port
    ))
    .await
    .map_err(|original| Error::ProxyIoError(original))?;
    let io = TokioIo::new(tcp);

    let (mut request_sender, proxy_connection) =
        conn::http1::handshake::<TokioIo<tokio::net::TcpStream>, String>(io).await?;
    let proxy_connection_task = tokio::spawn(proxy_connection.without_shutdown());

    let addr = format!("{domain}:{port}");
    let mut req_builder = Request::connect(addr);

    if let Some(auth) = proxy_config.auth {
        req_builder = req_builder.header("Proxy-Authorization", format!("Basic {}", auth.encode()));
    }

    let req = req_builder
        .body("".to_string())
        .map_err(|orig| Error::ProxyHttpError(orig))?;
    let res = request_sender.send_request(req).await?;
    if res.status().as_u16() < 200 || res.status().as_u16() >= 300 {
        return Err(Error::ProxyStatusError(res.status().as_u16()));
    }

    // expect is fine since it would only rely panics from within the tokio task (or a cancellation which does not happen)
    let proxy_connection = proxy_connection_task
        .await
        .expect("Panic in tokio task during websocket proxy initialization")?;

    let proxy_tcp_wrapper = proxy_connection.io;
    let proxied_tcp_socket = proxy_tcp_wrapper.into_inner();
    let (ws_stream, _) = client_async_tls_with_config(
        request,
        proxied_tcp_socket,
        config.map(Into::into),
        tls_connector,
    )
    .await?;

    Ok(ws_stream)
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

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::default().build()
}

#[derive(Default)]
pub struct Builder {
    tls_connector: Option<Connector>,
    proxy_configuration: Option<ProxyConfiguration>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            tls_connector: None,
            proxy_configuration: None,
        }
    }

    pub fn tls_connector(mut self, connector: Connector) -> Self {
        self.tls_connector.replace(connector);
        self
    }

    pub fn proxy_configuration(mut self, proxy_configuration: ProxyConfiguration) -> Self {
        self.proxy_configuration.replace(proxy_configuration);
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        PluginBuilder::new("websocket")
            .invoke_handler(tauri::generate_handler![connect, send])
            .setup(|app| {
                app.manage(ConnectionManager::default());
                app.manage(TlsConnector(Mutex::new(self.tls_connector)));
                app.manage(ProxyConfigurationInternal(Mutex::new(
                    self.proxy_configuration,
                )));

                Ok(())
            })
            .build()
    }
}

pub async fn reconfigure_proxy(app: &AppHandle, proxy_config: Option<ProxyConfiguration>) {
    if let Some(state) = app.try_state::<ProxyConfigurationInternal>() {
        if let Some(proxy_config) = proxy_config {
            state.0.lock().await.replace(proxy_config);
        } else {
            state.0.lock().await.take();
        }
    }
}

pub async fn reconfigure_tls_connector(app: &AppHandle, tls_connector: Option<Connector>) {
    if let Some(state) = app.try_state::<TlsConnector>() {
        if let Some(tls_connector) = tls_connector {
            state.0.lock().await.replace(tls_connector);
        } else {
            state.0.lock().await.take();
        }
    }
}

// Copied from tokio-tungstenite internal function (tokio-tungstenite/src/lib.rs) with the same name
// Get a domain from an URL.
#[inline]
fn domain(
    request: &tokio_tungstenite::tungstenite::handshake::client::Request,
) -> tokio_tungstenite::tungstenite::Result<String, tokio_tungstenite::tungstenite::Error> {
    match request.uri().host() {
        // rustls expects IPv6 addresses without the surrounding [] brackets
        #[cfg(feature = "__rustls-tls")]
        Some(d) if d.starts_with('[') && d.ends_with(']') => Ok(d[1..d.len() - 1].to_string()),
        Some(d) => Ok(d.to_string()),
        None => Err(tokio_tungstenite::tungstenite::Error::Url(
            tokio_tungstenite::tungstenite::error::UrlError::NoHostName,
        )),
    }
}
