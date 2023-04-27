use std::{collections::HashMap, path::PathBuf, time::Duration};

use reqwest::{header, Method, Url};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize)]
#[serde(untagged)]
enum SerdeDuration {
    Seconds(u64),
    Duration(Duration),
}

fn deserialize_duration<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<Duration>, D::Error> {
    if let Some(duration) = Option::<SerdeDuration>::deserialize(deserializer)? {
        Ok(Some(match duration {
            SerdeDuration::Seconds(s) => Duration::from_secs(s),
            SerdeDuration::Duration(d) => d,
        }))
    } else {
        Ok(None)
    }
}

/// The builder of [`Client`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientBuilder {
    /// Max number of redirections to follow.
    pub max_redirections: Option<usize>,
    /// Connect timeout for the request.
    #[serde(deserialize_with = "deserialize_duration", default)]
    pub connect_timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Builds the Client.
    pub fn build(self) -> crate::Result<Client> {
        let mut client_builder = reqwest::Client::builder();

        if let Some(max_redirections) = self.max_redirections {
            client_builder = client_builder.redirect(if max_redirections == 0 {
                reqwest::redirect::Policy::none()
            } else {
                reqwest::redirect::Policy::limited(max_redirections)
            });
        }

        if let Some(connect_timeout) = self.connect_timeout {
            client_builder = client_builder.connect_timeout(connect_timeout);
        }

        let client = client_builder.build()?;
        Ok(Client(client))
    }
}

/// The HTTP client based on [`reqwest`].
#[derive(Debug, Clone)]
pub struct Client(reqwest::Client);

impl Client {
    /// Executes an HTTP request
    ///
    /// # Examples
    pub async fn send(&self, mut request: HttpRequestBuilder) -> crate::Result<Response> {
        let method = Method::from_bytes(request.method.to_uppercase().as_bytes())?;

        let mut request_builder = self.0.request(method, request.url.as_str());

        if let Some(query) = request.query {
            request_builder = request_builder.query(&query);
        }

        if let Some(timeout) = request.timeout {
            request_builder = request_builder.timeout(timeout);
        }

        if let Some(body) = request.body {
            request_builder = match body {
                Body::Bytes(data) => request_builder.body(bytes::Bytes::from(data)),
                Body::Text(text) => request_builder.body(bytes::Bytes::from(text)),
                Body::Json(json) => request_builder.json(&json),
                Body::Form(form_body) => {
                    #[allow(unused_variables)]
                    fn send_form(
                        request_builder: reqwest::RequestBuilder,
                        headers: &mut Option<HeaderMap>,
                        form_body: FormBody,
                    ) -> crate::Result<reqwest::RequestBuilder> {
                        #[cfg(feature = "http-multipart")]
                        if matches!(
                            headers
                                .as_ref()
                                .and_then(|h| h.0.get("content-type"))
                                .map(|v| v.as_bytes()),
                            Some(b"multipart/form-data")
                        ) {
                            // the Content-Type header will be set by reqwest in the `.multipart` call
                            headers.as_mut().map(|h| h.0.remove("content-type"));
                            let mut multipart = reqwest::multipart::Form::new();

                            for (name, part) in form_body.0 {
                                let part = match part {
                                    FormPart::File {
                                        file,
                                        mime,
                                        file_name,
                                    } => {
                                        let bytes: Vec<u8> = file.try_into()?;
                                        let mut part = reqwest::multipart::Part::bytes(bytes);
                                        if let Some(mime) = mime {
                                            part = part.mime_str(&mime)?;
                                        }
                                        if let Some(file_name) = file_name {
                                            part = part.file_name(file_name);
                                        }
                                        part
                                    }
                                    FormPart::Text(value) => reqwest::multipart::Part::text(value),
                                };

                                multipart = multipart.part(name, part);
                            }

                            return Ok(request_builder.multipart(multipart));
                        }

                        let mut form = Vec::new();
                        for (name, part) in form_body.0 {
                            match part {
                                FormPart::File { file, .. } => {
                                    let bytes: Vec<u8> = file.try_into()?;
                                    form.push((name, serde_json::to_string(&bytes)?))
                                }
                                FormPart::Text(value) => form.push((name, value)),
                            }
                        }
                        Ok(request_builder.form(&form))
                    }
                    send_form(request_builder, &mut request.headers, form_body)?
                }
            };
        }

        if let Some(headers) = request.headers {
            request_builder = request_builder.headers(headers.0);
        }

        let http_request = request_builder.build()?;

        let response = self.0.execute(http_request).await?;

        Ok(Response(
            request.response_type.unwrap_or(ResponseType::Json),
            response,
        ))
    }
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
#[non_exhaustive]
/// The HTTP response type.
pub enum ResponseType {
    /// Read the response as JSON
    Json = 1,
    /// Read the response as text
    Text,
    /// Read the response as binary
    Binary,
}

#[derive(Debug)]
pub struct Response(ResponseType, reqwest::Response);

impl Response {
    /// Reads the response.
    ///
    /// Note that the body is serialized to a [`Value`].
    pub async fn read(self) -> crate::Result<ResponseData> {
        let url = self.1.url().clone();

        let mut headers = HashMap::new();
        let mut raw_headers = HashMap::new();
        for (name, value) in self.1.headers() {
            headers.insert(
                name.as_str().to_string(),
                String::from_utf8(value.as_bytes().to_vec())?,
            );
            raw_headers.insert(
                name.as_str().to_string(),
                self.1
                    .headers()
                    .get_all(name)
                    .into_iter()
                    .map(|v| String::from_utf8(v.as_bytes().to_vec()).map_err(Into::into))
                    .collect::<crate::Result<Vec<String>>>()?,
            );
        }
        let status = self.1.status().as_u16();

        let data = match self.0 {
            ResponseType::Json => self.1.json().await?,
            ResponseType::Text => Value::String(self.1.text().await?),
            ResponseType::Binary => serde_json::to_value(&self.1.bytes().await?)?,
        };

        Ok(ResponseData {
            url,
            status,
            headers,
            raw_headers,
            data,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResponseData {
    /// Response URL. Useful if it followed redirects.
    pub url: Url,
    /// Response status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response raw headers.
    pub raw_headers: HashMap<String, Vec<String>>,
    /// Response data.
    pub data: Value,
}

/// A file path or contents.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum FilePart {
    /// File path.
    Path(PathBuf),
    /// File contents.
    Contents(Vec<u8>),
}

impl TryFrom<FilePart> for Vec<u8> {
    type Error = crate::Error;
    fn try_from(file: FilePart) -> crate::Result<Self> {
        let bytes = match file {
            FilePart::Path(path) => std::fs::read(path)?,
            FilePart::Contents(bytes) => bytes,
        };
        Ok(bytes)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum FormPart {
    /// A string value.
    Text(String),
    /// A file value.
    #[serde(rename_all = "camelCase")]
    File {
        /// File path or content.
        file: FilePart,
        /// Mime type of this part.
        /// Only used when the `Content-Type` header is set to `multipart/form-data`.
        mime: Option<String>,
        /// File name.
        /// Only used when the `Content-Type` header is set to `multipart/form-data`.
        file_name: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct FormBody(pub(crate) HashMap<String, FormPart>);

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum Body {
    Form(FormBody),
    Json(Value),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Default)]
pub struct HeaderMap(header::HeaderMap);

impl<'de> Deserialize<'de> for HeaderMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestBuilder {
    /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
    pub method: String,
    /// The request URL
    pub url: Url,
    /// The request query params
    pub query: Option<HashMap<String, String>>,
    /// The request headers
    pub headers: Option<HeaderMap>,
    /// The request body
    pub body: Option<Body>,
    /// Timeout for the whole request
    #[serde(deserialize_with = "deserialize_duration", default)]
    pub timeout: Option<Duration>,
    /// The response type (defaults to Json)
    pub response_type: Option<ResponseType>,
}
