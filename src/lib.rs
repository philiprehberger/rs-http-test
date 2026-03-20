//! Declarative HTTP API integration testing framework with fluent assertions.
//!
//! This crate provides a builder-based API for constructing HTTP requests and
//! asserting properties of the responses. It is designed for integration tests
//! against real or mock HTTP servers.
//!
//! # Quick start
//!
//! ```no_run
//! use philiprehberger_http_test::get;
//!
//! let response = get("https://httpbin.org/get").send().unwrap();
//! response.assert_ok();
//! ```

use std::fmt;
use std::time::Duration;

/// Error type for HTTP test operations.
#[derive(Debug)]
pub enum HttpTestError {
    /// The HTTP request failed to execute.
    RequestFailed(String),
    /// An assertion on the response did not hold.
    AssertionFailed {
        /// The expected value.
        expected: String,
        /// The actual value.
        actual: String,
        /// Additional context describing the assertion.
        context: String,
    },
    /// A JSON path could not be resolved.
    JsonPathError(String),
    /// A connection to the server could not be established.
    ConnectionError(String),
}

impl fmt::Display for HttpTestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpTestError::RequestFailed(msg) => write!(f, "request failed: {msg}"),
            HttpTestError::AssertionFailed {
                expected,
                actual,
                context,
            } => write!(
                f,
                "assertion failed ({context}): expected {expected}, got {actual}"
            ),
            HttpTestError::JsonPathError(msg) => write!(f, "JSON path error: {msg}"),
            HttpTestError::ConnectionError(msg) => write!(f, "connection error: {msg}"),
        }
    }
}

impl std::error::Error for HttpTestError {}

/// A builder for constructing an HTTP test request.
pub struct TestRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    query: Vec<(String, String)>,
    timeout: Option<Duration>,
}

/// Create a GET request to the given URL.
pub fn get(url: &str) -> TestRequest {
    TestRequest::new("GET", url)
}

/// Create a POST request to the given URL.
pub fn post(url: &str) -> TestRequest {
    TestRequest::new("POST", url)
}

/// Create a PUT request to the given URL.
pub fn put(url: &str) -> TestRequest {
    TestRequest::new("PUT", url)
}

/// Create a DELETE request to the given URL.
pub fn delete(url: &str) -> TestRequest {
    TestRequest::new("DELETE", url)
}

/// Create a PATCH request to the given URL.
pub fn patch(url: &str) -> TestRequest {
    TestRequest::new("PATCH", url)
}

impl TestRequest {
    fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
            query: Vec::new(),
            timeout: None,
        }
    }

    /// Add a header to the request.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    /// Set the `Authorization: Bearer <token>` header.
    pub fn bearer_token(self, token: &str) -> Self {
        self.header("Authorization", &format!("Bearer {token}"))
    }

    /// Set the `Authorization: Basic <credentials>` header using base64-encoded `user:pass`.
    pub fn basic_auth(self, user: &str, pass: &str) -> Self {
        use std::io::Write;
        let mut buf = Vec::new();
        write!(buf, "{user}:{pass}").unwrap();
        let encoded = base64_encode(&buf);
        self.header("Authorization", &format!("Basic {encoded}"))
    }

    /// Add a query parameter to the request URL.
    pub fn query(mut self, key: &str, value: &str) -> Self {
        self.query.push((key.to_string(), value.to_string()));
        self
    }

    /// Set a JSON body and the `Content-Type: application/json` header.
    pub fn json_body(mut self, value: &serde_json::Value) -> Self {
        self.body = Some(value.to_string());
        self.headers
            .push(("Content-Type".to_string(), "application/json".to_string()));
        self
    }

    /// Set the raw request body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Execute the request and return the response.
    pub fn send(&self) -> Result<TestResponse, HttpTestError> {
        let mut url = self.url.clone();

        if !self.query.is_empty() {
            let sep = if url.contains('?') { '&' } else { '?' };
            let params: Vec<String> = self
                .query
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            url = format!("{url}{sep}{}", params.join("&"));
        }

        let client_builder = reqwest::blocking::ClientBuilder::new();
        let client_builder = if let Some(t) = self.timeout {
            client_builder.timeout(t)
        } else {
            client_builder
        };

        let client = client_builder
            .build()
            .map_err(|e| HttpTestError::ConnectionError(e.to_string()))?;

        let mut request = match self.method.as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            other => {
                return Err(HttpTestError::RequestFailed(format!(
                    "unsupported method: {other}"
                )))
            }
        };

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        if let Some(body) = &self.body {
            request = request.body(body.clone());
        }

        let response = request
            .send()
            .map_err(|e| HttpTestError::ConnectionError(e.to_string()))?;

        let status = response.status().as_u16();
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response
            .text()
            .map_err(|e| HttpTestError::RequestFailed(e.to_string()))?;

        Ok(TestResponse {
            status,
            headers,
            body,
        })
    }
}

/// The response from an HTTP test request, with assertion methods for validation.
pub struct TestResponse {
    /// The HTTP status code.
    pub status: u16,
    /// The response headers as key-value pairs.
    pub headers: Vec<(String, String)>,
    /// The response body as a string.
    pub body: String,
}

impl TestResponse {
    /// Assert that the status code matches the expected value.
    pub fn assert_status(&self, expected: u16) -> &Self {
        assert_eq!(
            self.status, expected,
            "expected status {expected}, got {}",
            self.status
        );
        self
    }

    /// Assert that the status code is in the 2xx range.
    pub fn assert_ok(&self) -> &Self {
        assert!(
            (200..300).contains(&self.status),
            "expected 2xx status, got {}",
            self.status
        );
        self
    }

    /// Assert that the status code is in the 3xx range.
    pub fn assert_redirect(&self) -> &Self {
        assert!(
            (300..400).contains(&self.status),
            "expected 3xx status, got {}",
            self.status
        );
        self
    }

    /// Assert that the status code is in the 4xx range.
    pub fn assert_client_error(&self) -> &Self {
        assert!(
            (400..500).contains(&self.status),
            "expected 4xx status, got {}",
            self.status
        );
        self
    }

    /// Assert that the status code is in the 5xx range.
    pub fn assert_server_error(&self) -> &Self {
        assert!(
            (500..600).contains(&self.status),
            "expected 5xx status, got {}",
            self.status
        );
        self
    }

    /// Assert that a response header has the expected value (case-insensitive key match).
    pub fn assert_header(&self, key: &str, value: &str) -> &Self {
        let lower_key = key.to_lowercase();
        let found = self
            .headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower_key);
        match found {
            Some((_, v)) => assert_eq!(
                v, value,
                "header '{key}': expected '{value}', got '{v}'"
            ),
            None => panic!("header '{key}' not found in response"),
        }
        self
    }

    /// Assert that a response header exists (case-insensitive key match).
    pub fn assert_header_exists(&self, key: &str) -> &Self {
        let lower_key = key.to_lowercase();
        assert!(
            self.headers.iter().any(|(k, _)| k.to_lowercase() == lower_key),
            "expected header '{key}' to exist"
        );
        self
    }

    /// Assert that the response body contains the given substring.
    pub fn assert_body_contains(&self, substring: &str) -> &Self {
        assert!(
            self.body.contains(substring),
            "expected body to contain '{substring}', body was: {}",
            truncate_for_display(&self.body, 200)
        );
        self
    }

    /// Assert that the response body equals the expected string exactly.
    pub fn assert_body_equals(&self, expected: &str) -> &Self {
        assert_eq!(
            self.body, expected,
            "expected body to equal '{expected}', got: {}",
            truncate_for_display(&self.body, 200)
        );
        self
    }

    /// Assert that a value at the given JSON path matches the expected value.
    ///
    /// Supports dot-separated keys and array indices:
    /// - `"name"` — top-level field
    /// - `"data.count"` — nested field
    /// - `"users[0].name"` — array index then field
    pub fn assert_json_path(&self, path: &str, expected: &serde_json::Value) -> &Self {
        let json: serde_json::Value = serde_json::from_str(&self.body).unwrap_or_else(|e| {
            panic!("failed to parse response body as JSON: {e}");
        });

        let actual = resolve_json_path(&json, path);

        match actual {
            Some(val) => assert_eq!(
                val, expected,
                "JSON path '{path}': expected {expected}, got {val}"
            ),
            None => panic!("JSON path '{path}' not found in response body"),
        }
        self
    }

    /// Parse the response body as JSON.
    pub fn json(&self) -> Result<serde_json::Value, HttpTestError> {
        serde_json::from_str(&self.body)
            .map_err(|e| HttpTestError::JsonPathError(format!("failed to parse JSON: {e}")))
    }
}

/// Resolve a simple JSON path expression against a JSON value.
///
/// Supports dot-separated keys with optional array indices, e.g.:
/// - `"name"` -> `value["name"]`
/// - `"data.count"` -> `value["data"]["count"]`
/// - `"users[0].name"` -> `value["users"][0]["name"]`
fn resolve_json_path<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    let segments = parse_path_segments(path);
    let mut current = value;

    for segment in &segments {
        match segment {
            PathSegment::Key(key) => {
                current = current.get(key.as_str())?;
            }
            PathSegment::Index(idx) => {
                current = current.get(*idx)?;
            }
        }
    }

    Some(current)
}

#[derive(Debug)]
enum PathSegment {
    Key(String),
    Index(usize),
}

/// Parse a JSON path string like `"users[0].name"` into segments.
fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();

    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some(bracket_pos) = part.find('[') {
            let key = &part[..bracket_pos];
            if !key.is_empty() {
                segments.push(PathSegment::Key(key.to_string()));
            }
            // Parse all bracket indices, e.g. "[0][1]"
            let rest = &part[bracket_pos..];
            let mut remaining = rest;
            while let Some(start) = remaining.find('[') {
                if let Some(end) = remaining.find(']') {
                    if let Ok(idx) = remaining[start + 1..end].parse::<usize>() {
                        segments.push(PathSegment::Index(idx));
                    }
                    remaining = &remaining[end + 1..];
                } else {
                    break;
                }
            }
        } else {
            segments.push(PathSegment::Key(part.to_string()));
        }
    }

    segments
}

/// Simple base64 encoding (no external dependency needed for this).
fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < input.len() {
        let b0 = input[i] as u32;
        let b1 = if i + 1 < input.len() { input[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if i + 1 < input.len() {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < input.len() {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

fn truncate_for_display(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::thread::{self, JoinHandle};

    /// Start a minimal HTTP test server that serves a canned response.
    /// Returns the base URL and a join handle for the server thread.
    fn start_test_server(
        status: u16,
        body: &str,
        headers: Vec<(&str, &str)>,
    ) -> (String, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}");
        let body = body.to_string();
        let headers: Vec<(String, String)> = headers
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());

            // Read the request line and headers
            let mut request_lines = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                request_lines.push(line);
            }

            // Read body if Content-Length is present
            let content_length: usize = request_lines
                .iter()
                .find(|l| l.to_lowercase().starts_with("content-length:"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0);

            if content_length > 0 {
                let mut body_buf = vec![0u8; content_length];
                reader.read_exact(&mut body_buf).ok();
            }

            let status_text = match status {
                200 => "OK",
                201 => "Created",
                301 => "Moved Permanently",
                302 => "Found",
                400 => "Bad Request",
                401 => "Unauthorized",
                404 => "Not Found",
                500 => "Internal Server Error",
                _ => "Unknown",
            };

            let mut response = format!("HTTP/1.1 {status} {status_text}\r\n");
            response.push_str(&format!("Content-Length: {}\r\n", body.len()));
            for (k, v) in &headers {
                response.push_str(&format!("{k}: {v}\r\n"));
            }
            response.push_str("\r\n");
            response.push_str(&body);

            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });

        (url, handle)
    }

    #[test]
    fn test_get_assert_status() {
        let (url, handle) = start_test_server(200, "hello", vec![]);
        let response = get(&url).send().unwrap();
        response.assert_status(200);
        handle.join().unwrap();
    }

    #[test]
    fn test_post_with_json_body() {
        let (url, handle) = start_test_server(201, r#"{"id":1}"#, vec![("Content-Type", "application/json")]);
        let response = post(&url)
            .json_body(&json!({"name": "Alice"}))
            .send()
            .unwrap();
        response.assert_status(201);
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_ok() {
        let (url, handle) = start_test_server(200, "ok", vec![]);
        let response = get(&url).send().unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_client_error() {
        let (url, handle) = start_test_server(404, "not found", vec![]);
        let response = get(&url).send().unwrap();
        response.assert_client_error();
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_header() {
        let (url, handle) = start_test_server(200, "ok", vec![("X-Custom", "test-value")]);
        let response = get(&url).send().unwrap();
        response
            .assert_status(200)
            .assert_header("X-Custom", "test-value");
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_header_exists() {
        let (url, handle) = start_test_server(200, "ok", vec![("X-Request-Id", "abc123")]);
        let response = get(&url).send().unwrap();
        response.assert_header_exists("X-Request-Id");
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_body_contains() {
        let (url, handle) = start_test_server(200, "hello world", vec![]);
        let response = get(&url).send().unwrap();
        response.assert_body_contains("world");
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_body_equals() {
        let (url, handle) = start_test_server(200, "exact match", vec![]);
        let response = get(&url).send().unwrap();
        response.assert_body_equals("exact match");
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_json_path_nested() {
        let body = r#"{"data":{"users":[{"name":"Alice"},{"name":"Bob"}]}}"#;
        let (url, handle) = start_test_server(200, body, vec![("Content-Type", "application/json")]);
        let response = get(&url).send().unwrap();
        response
            .assert_json_path("data.users[0].name", &json!("Alice"))
            .assert_json_path("data.users[1].name", &json!("Bob"));
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_json_path_simple() {
        let body = r#"{"count":42,"active":true}"#;
        let (url, handle) = start_test_server(200, body, vec![]);
        let response = get(&url).send().unwrap();
        response
            .assert_json_path("count", &json!(42))
            .assert_json_path("active", &json!(true));
        handle.join().unwrap();
    }

    #[test]
    fn test_bearer_token() {
        // We can verify the token is set by checking the request reaches the server.
        // The test server doesn't validate the token, but we verify the builder works.
        let (url, handle) = start_test_server(200, "ok", vec![]);
        let response = get(&url).bearer_token("my-secret-token").send().unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_basic_auth() {
        let (url, handle) = start_test_server(200, "ok", vec![]);
        let response = get(&url).basic_auth("admin", "password").send().unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_query_params() {
        let (url, handle) = start_test_server(200, "ok", vec![]);
        let response = get(&url)
            .query("page", "1")
            .query("limit", "10")
            .send()
            .unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_put_request() {
        let (url, handle) = start_test_server(200, "updated", vec![]);
        let response = put(&url).body("data").send().unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_delete_request() {
        let (url, handle) = start_test_server(200, "", vec![]);
        let response = delete(&url).send().unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_patch_request() {
        let (url, handle) = start_test_server(200, "patched", vec![]);
        let response = patch(&url).body("partial").send().unwrap();
        response.assert_ok();
        handle.join().unwrap();
    }

    #[test]
    fn test_json_parsing() {
        let body = r#"{"key":"value"}"#;
        let (url, handle) = start_test_server(200, body, vec![]);
        let response = get(&url).send().unwrap();
        let json = response.json().unwrap();
        assert_eq!(json["key"], "value");
        handle.join().unwrap();
    }

    #[test]
    fn test_assert_redirect() {
        let response = TestResponse {
            status: 302,
            headers: vec![("location".to_string(), "https://example.com".to_string())],
            body: String::new(),
        };
        response.assert_redirect();
    }

    #[test]
    fn test_assert_server_error() {
        let (url, handle) = start_test_server(500, "error", vec![]);
        let response = get(&url).send().unwrap();
        response.assert_server_error();
        handle.join().unwrap();
    }

    #[test]
    fn test_chained_assertions() {
        let body = r#"{"status":"ok","count":5}"#;
        let (url, handle) = start_test_server(200, body, vec![("X-Api", "v1")]);
        let response = get(&url).send().unwrap();
        response
            .assert_ok()
            .assert_status(200)
            .assert_header("X-Api", "v1")
            .assert_body_contains("ok")
            .assert_json_path("count", &json!(5));
        handle.join().unwrap();
    }

    #[test]
    #[should_panic(expected = "expected status 201")]
    fn test_assert_status_fails() {
        let response = TestResponse {
            status: 200,
            headers: vec![],
            body: String::new(),
        };
        response.assert_status(201);
    }

    #[test]
    #[should_panic(expected = "expected 2xx status")]
    fn test_assert_ok_fails() {
        let response = TestResponse {
            status: 404,
            headers: vec![],
            body: String::new(),
        };
        response.assert_ok();
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"admin:password"), "YWRtaW46cGFzc3dvcmQ=");
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_resolve_json_path() {
        let data = json!({
            "a": {
                "b": [1, 2, 3],
                "c": "hello"
            }
        });
        assert_eq!(resolve_json_path(&data, "a.c"), Some(&json!("hello")));
        assert_eq!(resolve_json_path(&data, "a.b[1]"), Some(&json!(2)));
        assert_eq!(resolve_json_path(&data, "a.missing"), None);
    }

    #[test]
    fn test_error_display() {
        let err = HttpTestError::RequestFailed("timeout".to_string());
        assert_eq!(format!("{err}"), "request failed: timeout");

        let err = HttpTestError::ConnectionError("refused".to_string());
        assert_eq!(format!("{err}"), "connection error: refused");

        let err = HttpTestError::JsonPathError("invalid".to_string());
        assert_eq!(format!("{err}"), "JSON path error: invalid");

        let err = HttpTestError::AssertionFailed {
            expected: "200".to_string(),
            actual: "404".to_string(),
            context: "status".to_string(),
        };
        assert_eq!(
            format!("{err}"),
            "assertion failed (status): expected 200, got 404"
        );
    }
}
