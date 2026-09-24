//! HTTP client for the Spaceship API, with status-to-ErrorCode translation.

use std::time::Duration;

use serde_json::Value;
use ureq::Agent;
use ureq::http::Response;

use crate::error::{Error, ErrorCode, Result};

const DEFAULT_BASE: &str = "https://spaceship.dev/api/v1/";
const MAX_BODY: u64 = 512 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: Agent,
    base: String,
    api_key: String,
    api_secret: String,
    verbose: bool,
}

enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Client {
    pub fn new(api_key: &str, api_secret: &str, verbose: bool) -> Client {
        let agent: Agent = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(100)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("spaceship-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();

        let mut base = std::env::var("SPACESHIP_API_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BASE.to_string());
        if !base.ends_with('/') {
            base.push('/');
        }

        Client { agent, base, api_key: api_key.to_string(), api_secret: api_secret.to_string(), verbose }
    }

    pub fn get(&self, path: &str) -> Result<Value> {
        self.send(Method::Get, path, None)
    }

    pub fn post(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        self.send(Method::Post, path, body)
    }

    pub fn put(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        self.send(Method::Put, path, body)
    }

    pub fn patch(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        self.send(Method::Patch, path, body)
    }

    pub fn delete(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        self.send(Method::Delete, path, body)
    }

    fn send(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let clean_path = path.trim_start_matches('/');
        let url = format!("{}{}", self.base, clean_path);

        if self.verbose {
            let m = match method {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Patch => "PATCH",
                Method::Delete => "DELETE",
            };
            eprintln!(">> {m} {url}");
        }

        macro_rules! headers {
            ($req:expr) => {{
                $req.header("X-API-Key", &self.api_key)
                    .header("X-API-Secret", &self.api_secret)
                    .header("Accept", "application/json")
            }};
        }

        let result = match (method, body) {
            (Method::Get, _) => headers!(self.agent.get(&url)).call(),
            (Method::Delete, None) => headers!(self.agent.delete(&url)).call(),
            (Method::Delete, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                headers!(self.agent.delete(&url))
                    .force_send_body()
                    .header("Content-Type", "application/json")
                    .send(&json[..])
            }
            (m, Some(b)) => {
                let json = serde_json::to_vec(b).expect("a Value always serializes");
                let req = match m {
                    Method::Post => self.agent.post(&url),
                    Method::Put => self.agent.put(&url),
                    _ => self.agent.patch(&url),
                };
                headers!(req).header("Content-Type", "application/json").send(&json[..])
            }
            (m, None) => {
                let req = match m {
                    Method::Post => self.agent.post(&url),
                    Method::Put => self.agent.put(&url),
                    _ => self.agent.patch(&url),
                };
                headers!(req).send_empty()
            }
        };

        let response = result.map_err(transport_error)?;
        self.handle_response(response)
    }

    fn handle_response(&self, mut response: Response<ureq::Body>) -> Result<Value> {
        let status = response.status().as_u16();

        let operation_id =
            response.headers().get("spaceship-operation-id").and_then(|h| h.to_str().ok()).map(str::to_string);

        let async_op_id = response
            .headers()
            .get("spaceship-async-operationid")
            .or_else(|| response.headers().get("spaceship-async-operation-id"))
            .and_then(|h| h.to_str().ok())
            .map(str::to_string);

        if self.verbose {
            let op_str = operation_id.as_deref().map(|id| format!(" (operation {id})")).unwrap_or_default();
            eprintln!("<< {status} {op_str}");
        }

        let bytes = response.body_mut().with_config().limit(MAX_BODY).read_to_vec().map_err(transport_error)?;
        let body_str = String::from_utf8_lossy(&bytes).trim().to_string();

        if !(200..300).contains(&status) {
            let retry_after =
                response.headers().get("retry-after").and_then(|h| h.to_str().ok()).and_then(|s| s.parse::<u64>().ok());
            return Err(status_error(status, &body_str, retry_after));
        }

        if body_str.is_empty() {
            let mut map = serde_json::Map::new();
            map.insert("status".into(), Value::String("ok".into()));
            map.insert("success".into(), Value::Bool(true));
            if let Some(op) = async_op_id {
                map.insert("asyncOperationId".into(), Value::String(op));
            }
            return Ok(Value::Object(map));
        }

        let mut parsed = match serde_json::from_str::<Value>(&body_str) {
            Ok(v) => v,
            Err(_) => Value::String(body_str),
        };

        if let (Value::Object(map), Some(op)) = (&mut parsed, async_op_id)
            && !map.contains_key("asyncOperationId")
        {
            map.insert("asyncOperationId".into(), Value::String(op));
        }

        Ok(parsed)
    }
}

fn transport_error(e: ureq::Error) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::new(ErrorCode::Network, "The request timed out.").fix("Retry once, then stop.")
        }
        other => Error::new(ErrorCode::Network, "Could not reach the Spaceship API.")
            .detail(other.to_string())
            .fix("Retry once, then stop."),
    }
}

pub fn status_error(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    let msg = try_extract_error_message(body).unwrap_or_else(|| format!("API error: HTTP {status}"));

    let mut detail = format!("HTTP {status}");
    if !body.is_empty() {
        detail.push_str(": ");
        detail.push_str(body);
    }

    match status {
        401 => Error::new(ErrorCode::AuthRequired, msg)
            .detail(detail)
            .fix("Check your API key and secret: spaceship accounts test <name> or set SPACESHIP_API_KEY and SPACESHIP_API_SECRET."),
        403 => Error::new(ErrorCode::AuthRequired, msg)
            .detail(detail)
            .fix("Verify that your API key has the required scope for this endpoint."),
        404 => Error::new(ErrorCode::NotFound, msg).detail(detail),
        429 => {
            let retry_str = retry_after.map(|s| format!(" Retry after {s}s.")).unwrap_or_default();
            Error::new(ErrorCode::RateLimited, format!("Rate limited (429).{retry_str}"))
                .detail(detail)
                .fix("Back off before retrying.")
        }
        400 | 422 => Error::new(ErrorCode::InvalidInput, msg).detail(detail),
        s if s >= 500 => Error::new(ErrorCode::Network, msg)
            .detail(detail)
            .fix("The Spaceship API returned a server error. Retry once, then stop."),
        _ => Error::new(ErrorCode::Error, msg).detail(detail),
    }
}

fn try_extract_error_message(body: &str) -> Option<String> {
    let root: Value = serde_json::from_str(body).ok()?;
    let obj = root.as_object()?;

    let mut text: Option<String> = None;
    for key in &["message", "error", "detail", "title"] {
        if let Some(v) = obj.get(*key).and_then(Value::as_str) {
            text = Some(v.to_string());
            break;
        }
    }

    let mut details: Vec<String> = Vec::new();

    // Spaceship validation errors: {"detail": "...", "data": [{"field": "...", "details": "..."}]}
    if let Some(data) = obj.get("data").and_then(Value::as_array) {
        for item in data {
            if let Some(item_obj) = item.as_object() {
                let field = item_obj.get("field").and_then(Value::as_str);
                let message = item_obj.get("details").and_then(Value::as_str);
                if let Some(m) = message.filter(|m| !m.trim().is_empty()) {
                    if let Some(f) = field.filter(|f| !f.trim().is_empty()) {
                        details.push(format!("{f}: {m}"));
                    } else {
                        details.push(m.to_string());
                    }
                }
            }
        }
    }

    // ASP.NET style errors: {"title": "...", "errors": {"field": ["msg", ...]}}
    if let Some(errors) = obj.get("errors").and_then(Value::as_object) {
        for (field, val) in errors {
            if let Some(arr) = val.as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        details.push(format!("{field}: {s}"));
                    }
                }
            } else if let Some(s) = val.as_str() {
                details.push(format!("{field}: {s}"));
            }
        }
    }

    if !details.is_empty() {
        let base = text.unwrap_or_else(|| "Validation error".into());
        Some(format!("{base} - {}", details.join("; ")))
    } else {
        text
    }
}

/// Percent-encodes one path segment.
pub fn seg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Builds `?a=1&b=2` from query parameters, skipping None or empty strings.
pub fn query(pairs: &[(&str, Option<String>)]) -> String {
    let parts: Vec<String> = pairs
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()).map(|v| format!("{k}={}", seg(v))))
        .collect();
    if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
}
