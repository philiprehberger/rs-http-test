# rs-http-test

[![CI](https://github.com/philiprehberger/rs-http-test/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-http-test/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-http-test.svg)](https://crates.io/crates/philiprehberger-http-test)
[![License](https://img.shields.io/github/license/philiprehberger/rs-http-test)](LICENSE)
[![Sponsor](https://img.shields.io/badge/sponsor-GitHub%20Sponsors-ec6cb9)](https://github.com/sponsors/philiprehberger)

Declarative HTTP API integration testing framework with fluent assertions

## Installation

```toml
[dependencies]
philiprehberger-http-test = "0.1.3"
```

## Usage

```rust
use philiprehberger_http_test::get;

let response = get("https://httpbin.org/get").send().unwrap();
response.assert_ok();
```

### POST with JSON

```rust
use philiprehberger_http_test::post;
use serde_json::json;

let response = post("https://httpbin.org/post")
    .json_body(&json!({"name": "Alice"}))
    .bearer_token("my-token")
    .send()
    .unwrap();

response
    .assert_status(200)
    .assert_body_contains("Alice");
```

### JSON path assertions

```rust
response.assert_json_path("data.name", &json!("Alice"));
```

## API

| Function / Type | Description |
|----------------|-------------|
| `get(url)` | Create GET request |
| `post(url)` | Create POST request |
| `put(url)` | Create PUT request |
| `delete(url)` | Create DELETE request |
| `.header(k, v)` | Add request header |
| `.bearer_token(t)` | Set Bearer auth |
| `.json_body(val)` | Set JSON request body |
| `.query(k, v)` | Add query parameter |
| `.send()` | Execute the request |
| `.assert_status(code)` | Assert status code |
| `.assert_ok()` | Assert 2xx status |
| `.assert_json_path(path, val)` | Assert JSON field value |
| `.assert_body_contains(s)` | Assert body contains string |

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## License

MIT
