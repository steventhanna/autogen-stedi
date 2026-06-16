# autogen-stedi

[![CI](https://github.com/steventhanna/autogen-stedi/actions/workflows/ci.yml/badge.svg)](https://github.com/steventhanna/autogen-stedi/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/autogen-stedi.svg)](https://crates.io/crates/autogen-stedi)
[![docs.rs](https://img.shields.io/docsrs/autogen-stedi)](https://docs.rs/autogen-stedi)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license)

Auto-generated, strongly-typed, async Rust client for the [Stedi APIs](https://www.stedi.com/docs).

Every request/response type and API method is generated directly from Stedi's public
[OpenAPI specs](https://github.com/Stedi/openapi) with
[openapi-generator](https://openapi-generator.tech/), so the surface stays faithful to the APIs and
updates automatically when a spec changes. A thin hand-written `StediClient` adds API-key auth on
top.

- **Complete** — all seven Stedi APIs (Claims, Core, Enrollment, Event Destinations, Healthcare,
  Manager, Payers), every endpoint and model.
- **Async** — built on [`reqwest`](https://docs.rs/reqwest); works on any Tokio runtime.
- **Modular** — every API is a Cargo feature, so you compile only what you use.
- **Collision-free** — each API lives in its own module (`claims`, `healthcare`, …), so identically
  named models across specs never clash.
- **No magic** — generated code is committed; no build scripts, no proc-macros, no codegen at build time.

## Installation

```toml
[dependencies]
autogen-stedi = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Compile only the APIs you need (faster builds):

```toml
[dependencies]
autogen-stedi = { version = "0.1", default-features = false, features = ["healthcare", "native-tls"] }
```

> **Note:** with `default-features = false` you must enable a TLS backend — either `native-tls`
> or `rustls` — or HTTPS requests will fail at runtime.

## Quick Start

```rust,no_run
use autogen_stedi::StediClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = StediClient::new("your-api-key");

    // Each accessor returns a `Configuration` for that API, with the correct base URL and auth set.
    let healthcare = client.healthcare();

    // Pass it to any function in the matching `apis` module:
    // autogen_stedi::healthcare::apis::<some_api>::<operation>(&healthcare, ...).await?;
    let _ = healthcare;
    Ok(())
}
```

## The seven APIs

Each Stedi API is a top-level module and a Cargo feature. The client accessor returns that API's
generated `Configuration`:

| Feature | Module | Accessor | Base URL |
|---------|--------|----------|----------|
| `claims` | `claims` | `client.claims()` | `https://claims.us.stedi.com` |
| `core` | `core` | `client.core()` | `https://core.us.stedi.com` |
| `enrollment` | `enrollment` | `client.enrollment()` | `https://enrollments.us.stedi.com` |
| `event-destinations` | `event_destinations` | `client.event_destinations()` | `https://events.us.stedi.com` |
| `healthcare` | `healthcare` | `client.healthcare()` | `https://healthcare.us.stedi.com` |
| `manager` | `manager` | `client.manager()` | `https://manager.us.stedi.com` |
| `payers` | `payers` | `client.payers()` | `https://payers.us.stedi.com` |

**TLS backends** (one required): `native-tls` (default) or `rustls`.

## Authentication

All Stedi APIs authenticate with an API key sent in the `Authorization` header as `Key <api-key>`.
`StediClient` wires this into every service's `Configuration`:

```rust
use autogen_stedi::StediClient;

let client = StediClient::new("your-api-key");
```

The base URL (including the dated API version such as `/2024-04-01`) is baked in from each spec. To
point at a proxy or mock server, mutate `base_path` on the returned configuration:

```rust,no_run
# use autogen_stedi::StediClient;
let client = StediClient::new("your-api-key");
# #[cfg(feature = "healthcare")] {
let mut healthcare = client.healthcare();
healthcare.base_path = "http://localhost:8080".to_string();
# }
```

## Error Handling

Generated functions return `Result<T, apis::Error<E>>`, where `E` is the endpoint-specific error
enum. Each service module has its own `apis::Error`, distinguishing transport errors,
(de)serialization errors, and structured API error responses:

```rust,ignore
use autogen_stedi::healthcare::apis::Error;

match some_call(&config).await {
    Ok(resp) => { /* ... */ }
    Err(Error::ResponseError(e)) => eprintln!("API returned HTTP {}: {}", e.status, e.content),
    Err(Error::Reqwest(e)) => eprintln!("transport error: {e}"),
    Err(e) => eprintln!("other error: {e}"),
}
```

## How It's Generated

```bash
# Requires: brew install openapi-generator
./generate.sh
```

`generate.sh` fetches all seven specs from the
[Stedi OpenAPI repo](https://github.com/Stedi/openapi), records their combined SHA-256 in
`SPEC_HASH`, runs openapi-generator (rust + reqwest template) on each, and vendors the generated
`apis/` and `models/` into a per-service module under `src/<service>/`. Because the rust generator
emits absolute `crate::apis` / `crate::models` paths, the script rewrites them to
`crate::<service>::…` so the code compiles inside a submodule. Finally it runs `cargo check`. The run
is idempotent: a fresh generation reproduces the committed tree exactly.

Only the entry points are hand-written and protected from regeneration: `Cargo.toml`, `src/lib.rs`,
`src/client.rs`, plus `README.md` and `CLAUDE.md`. Everything under `src/<service>/` is generated —
**do not edit it by hand**; fix the spec upstream or adjust `generate.sh` instead.

### Staying in sync with the specs

The crate version is content-hash based, starting at `0.1.0`. A scheduled GitHub Action
(`update-spec.yml`) regenerates from the live specs **daily**; when the combined hash changes it
bumps the patch version and opens a PR. Merging that PR tags the release and publishes the new
version to crates.io (`tag-release.yml`).

Publishing requires a `CARGO_REGISTRY_TOKEN` repository secret (a crates.io API token). The first
`0.1.0` release should be published manually (`cargo publish`) to establish crate ownership;
automated patch releases flow through CI from then on.

## Why auto-generated?

Hand-written SDKs drift from the API and accumulate subtle type mismatches. Generating from the
specs keeps the client honest:

- **Always current** — a spec change becomes a PR, not a manually-tracked changelog entry.
- **Exhaustive** — every endpoint and model is present, not just the popular ones.
- **Auditable** — generated code is committed and reviewable; there's no build-time codegen to trust.

## License

MIT
