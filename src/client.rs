//! Minimal blocking Elasticsearch HTTP client.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::Value;
use ureq::http::Response;
use ureq::{Agent, Body};

pub struct Client {
    base: String,
    agent: Agent,
    auth: Option<String>,
    verbose: bool,
}

impl Client {
    pub fn new(host: &str, auth: Option<String>, verbose: bool) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .http_status_as_error(false)
            .build();
        Self {
            base: host.trim_end_matches('/').to_string(),
            agent: config.into(),
            auth,
            verbose,
        }
    }

    fn auth_header(&self) -> Option<String> {
        self.auth
            .as_ref()
            .map(|creds| format!("Basic {}", BASE64.encode(creds)))
    }

    /// GET `path` and decode the response body as JSON.
    pub fn get_json(&self, path: &str, params: &[(String, String)]) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.agent.get(&url);
        if let Some(header) = self.auth_header() {
            req = req.header("Authorization", header.as_str());
        }
        for (key, value) in params {
            req = req.query(key, value);
        }
        if self.verbose {
            eprintln!("> GET {url}");
        }
        let resp = req.call().with_context(|| format!("GET {url} failed"))?;
        let mut resp = self.check(resp)?;
        let value: Value = resp
            .body_mut()
            .read_json()
            .with_context(|| format!("GET {url}: invalid JSON response"))?;
        Ok(value)
    }

    /// GET `path` and return the response body as text (for `_cat` output).
    pub fn get_text(&self, path: &str, params: &[(String, String)]) -> Result<String> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.agent.get(&url);
        if let Some(header) = self.auth_header() {
            req = req.header("Authorization", header.as_str());
        }
        for (key, value) in params {
            req = req.query(key, value);
        }
        if self.verbose {
            eprintln!("> GET {url}");
        }
        let resp = req.call().with_context(|| format!("GET {url} failed"))?;
        let mut resp = self.check(resp)?;
        let text = resp
            .body_mut()
            .read_to_string()
            .with_context(|| format!("GET {url}: invalid response body"))?;
        Ok(text)
    }

    /// POST `path` with a JSON body and decode the response as JSON.
    pub fn post_json(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.agent.post(&url);
        if let Some(header) = self.auth_header() {
            req = req.header("Authorization", header.as_str());
        }
        if self.verbose {
            eprintln!("> POST {url}");
        }
        let resp = req
            .send_json(body)
            .with_context(|| format!("POST {url} failed"))?;
        let mut resp = self.check(resp)?;
        let value: Value = resp
            .body_mut()
            .read_json()
            .with_context(|| format!("POST {url}: invalid JSON response"))?;
        Ok(value)
    }

    fn check(&self, mut resp: Response<Body>) -> Result<Response<Body>> {
        if self.verbose {
            eprintln!("< {}", resp.status());
        }
        if resp.status().is_success() {
            return Ok(resp);
        }
        let status = resp.status();
        let body = resp.body_mut().read_to_string().unwrap_or_default();
        // Surface Elasticsearch's own error reason when present.
        let reason = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| {
                v.pointer("/error/reason")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| body.trim().to_string());
        Err(anyhow!("Elasticsearch error {status}: {reason}"))
    }
}
