use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::blocking::{Client as HttpClient, RequestBuilder};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{Error, Result};

pub const BASE_URL: &str = "https://hlwicpfwc.miit.gov.cn/icpproject_query/api/";
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(15);

const DEFAULT_HEADERS: [(&str, &str); 6] = [
    ("origin", "https://beian.miit.gov.cn"),
    ("referer", "https://beian.miit.gov.cn/"),
    (
        "user-agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36 Edg/146.0.0.0",
    ),
    ("accept", "application/json, text/plain, */*"),
    (
        "accept-language",
        "zh-CN,zh-HK;q=0.9,zh;q=0.8,en-US;q=0.7,en;q=0.6",
    ),
    ("connection", "keep-alive"),
];

/// Blocking API client with a cookie jar and sticky session headers.
pub struct Client {
    http: HttpClient,
    /// Lower-cased header name → value, sent with every request.
    headers: BTreeMap<String, String>,
}

impl Client {
    /// A zero `timeout` selects [`DEFAULT_HTTP_TIMEOUT`].
    pub fn new(timeout: Duration) -> Result<Self> {
        let timeout = if timeout.is_zero() {
            DEFAULT_HTTP_TIMEOUT
        } else {
            timeout
        };
        let http = HttpClient::builder()
            .timeout(timeout)
            .cookie_store(true)
            .build()?;
        let headers = DEFAULT_HEADERS
            .iter()
            .map(|&(key, value)| (key.to_owned(), value.to_owned()))
            .collect();

        Ok(Self { http, headers })
    }

    /// Sets a session header; an empty `value` removes it.
    pub fn set_header(&mut self, key: &str, value: &str) {
        let key = key.to_ascii_lowercase();
        if value.is_empty() {
            self.headers.remove(&key);
        } else {
            self.headers.insert(key, value.to_owned());
        }
    }

    pub fn set_token(&mut self, token: &str) {
        self.set_header("Token", token);
    }

    pub fn set_authorization(&mut self, authorization: &str) {
        self.set_header("Authorization", authorization);
    }

    /// Builds a request to `path` (relative to [`BASE_URL`] or absolute) carrying the session headers.
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        self.headers.iter().fold(
            self.http.request(method, resolve_url(path).as_ref()),
            |request, (key, value)| request.header(key.as_str(), value.as_str()),
        )
    }

    pub fn post_form_json<F, T>(&self, path: &str, form: &F) -> Result<T>
    where
        F: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        decode_json(&send(self.request(Method::POST, path).form(form))?)
    }

    pub fn post_json<P, T>(&self, path: &str, payload: &P) -> Result<T>
    where
        P: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        decode_json(&self.post_json_raw(path, payload)?)
    }

    pub fn post_json_raw<P>(&self, path: &str, payload: &P) -> Result<Vec<u8>>
    where
        P: Serialize + ?Sized,
    {
        send(self.request(Method::POST, path).json(payload))
    }
}

fn send(request: RequestBuilder) -> Result<Vec<u8>> {
    let response = request.send()?;
    let status = response.status();
    let body = Vec::from(response.bytes()?);

    if status != StatusCode::OK {
        return Err(Error::Status {
            status: status.as_u16(),
            body: lossy_trimmed(&body),
        });
    }
    Ok(body)
}

fn decode_json<T: DeserializeOwned>(body: &[u8]) -> Result<T> {
    serde_json::from_slice(body).map_err(|source| Error::Decode {
        source,
        body: lossy_trimmed(body),
    })
}

fn lossy_trimmed(body: &[u8]) -> String {
    String::from_utf8_lossy(body).trim().to_owned()
}

fn resolve_url(path: &str) -> Cow<'_, str> {
    if path.starts_with("http://") || path.starts_with("https://") {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(format!(
            "{BASE_URL}{}",
            path.strip_prefix('/').unwrap_or(path)
        ))
    }
}
