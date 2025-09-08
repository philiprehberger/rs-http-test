# rs-http-test

[![CI](https://github.com/philiprehberger/rs-http-test/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-http-test/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-http-test.svg)](https://crates.io/crates/philiprehberger-http-test)
[![Last updated](https://img.shields.io/github/last-commit/philiprehberger/rs-http-test)](https://github.com/philiprehberger/rs-http-test/commits/main)

Declarative HTTP API integration testing framework with fluent assertions

## Installation

```toml
[dependencies]
philiprehberger-http-test = "0.1.4"
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

## Support

If you find this project useful:

⭐ [Star the repo](https://github.com/philiprehberger/rs-http-test)

🐛 [Report issues](https://github.com/philiprehberger/rs-http-test/issues?q=is%3Aissue+is%3Aopen+label%3Abug)

💡 [Suggest features](https://github.com/philiprehberger/rs-http-test/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)

❤️ [Sponsor development](https://github.com/sponsors/philiprehberger)

🌐 [All Open Source Projects](https://philiprehberger.com/open-source-packages)

💻 [GitHub Profile](https://github.com/philiprehberger)

🔗 [LinkedIn Profile](https://www.linkedin.com/in/philiprehberger)

## License

[MIT](LICENSE)
