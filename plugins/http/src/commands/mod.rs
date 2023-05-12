use tauri::{path::SafePathBuf, AppHandle, Runtime, State};

use crate::{ClientId, Http};

mod client;
use client::{Body, ClientBuilder, FilePart, FormPart, HttpRequestBuilder, ResponseData};

pub use client::Client;

#[tauri::command]
pub async fn create_client<R: Runtime>(
    _app: AppHandle<R>,
    http: State<'_, Http<R>>,
    options: Option<ClientBuilder>,
) -> super::Result<ClientId> {
    let client = options.unwrap_or_default().build()?;
    let mut store = http.clients.lock().unwrap();
    let id = rand::random::<ClientId>();
    store.insert(id, client);
    Ok(id)
}

#[tauri::command]
pub async fn drop_client<R: Runtime>(
    _app: AppHandle<R>,
    http: State<'_, Http<R>>,
    client: ClientId,
) -> super::Result<()> {
    let mut store = http.clients.lock().unwrap();
    store.remove(&client);
    Ok(())
}

#[tauri::command]
pub async fn request<R: Runtime>(
    app: AppHandle<R>,
    http: State<'_, Http<R>>,
    client_id: ClientId,
    options: Box<HttpRequestBuilder>,
) -> super::Result<ResponseData> {
    use crate::Manager;
    if http.scope.is_allowed(&options.url) {
        let client = http
            .clients
            .lock()
            .unwrap()
            .get(&client_id)
            .ok_or_else(|| crate::Error::HttpClientNotInitialized)?
            .clone();
        let options = *options;
        if let Some(Body::Form(form)) = &options.body {
            for value in form.0.values() {
                if let FormPart::File {
                    file: FilePart::Path(path),
                    ..
                } = value
                {
                    if SafePathBuf::new(path.clone()).is_err() || !app.fs_scope().is_allowed(path) {
                        return Err(crate::Error::PathNotAllowed(path.clone()));
                    }
                }
            }
        }
        let response = client.send(options).await?;
        Ok(response.read().await?)
    } else {
        Err(crate::Error::UrlNotAllowed(options.url))
    }
}
