# Changelog

## 0.1.5 (2026-03-31)

- Standardize README to 3-badge format with emoji Support section
- Update CI checkout action to v5 for Node.js 24 compatibility

## 0.1.4 (2026-03-27)

- Add GitHub issue templates, PR template, and dependabot configuration
- Update README badges and add Support section

## 0.1.3 (2026-03-22)

- Fix CHANGELOG and CI compliance

## 0.1.2 (2026-03-20)

- Fix CI workflow to use env var for registry token

## 0.1.1 (2026-03-20)

- Re-release with registry token configured

## 0.1.0 (2026-03-19)

- Declarative HTTP request builder with `get()`, `post()`, `put()`, `delete()`, `patch()` constructors
- Fluent request configuration: headers, query params, JSON body, timeout
- Authentication helpers: `bearer_token()` and `basic_auth()`
- Chainable response assertions: `assert_status()`, `assert_ok()`, `assert_redirect()`, `assert_client_error()`, `assert_server_error()`
- Header assertions: `assert_header()`, `assert_header_exists()`
- Body assertions: `assert_body_contains()`, `assert_body_equals()`
- JSON path assertions with dot notation and array index support
- `HttpTestError` enum for structured error handling
