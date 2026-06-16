//! # autogen-stedi
//!
//! Auto-generated, strongly-typed, async Rust client for the
//! [Stedi APIs](https://www.stedi.com/docs).
//!
//! Every request/response type and API method is generated directly from Stedi's public
//! [OpenAPI specs](https://github.com/Stedi/openapi) with
//! [openapi-generator](https://openapi-generator.tech/), so the surface stays faithful to the
//! APIs and updates automatically when a spec changes. A thin hand-written [`StediClient`] adds
//! API-key authentication and hands you a per-service
//! [`Configuration`](crate::client::StediClient) with the correct base URL already set.
//!
//! ## Why one module per service?
//!
//! Stedi publishes **several independent APIs**, each with its own base URL
//! (`claims.us.stedi.com`, `healthcare.us.stedi.com`, …) and its own spec. Each is vendored into
//! its own top-level module — [`claims`], [`core`], [`enrollment`], [`event_destinations`],
//! [`healthcare`], [`manager`], [`payers`] — so their (otherwise colliding) model names stay
//! isolated. Every service is a Cargo feature: enabling only what you use skips compiling the rest
//! entirely, models included.
//!
//! ## Quick start
//!
//! ```no_run
//! use autogen_stedi::StediClient;
//!
//! # #[cfg(feature = "healthcare")]
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = StediClient::new("your-api-key");
//!
//!     // `client.healthcare()` returns a `Configuration` pointed at the Healthcare API,
//!     // ready to pass to any function in `autogen_stedi::healthcare::apis`.
//!     let _config = client.healthcare();
//!     Ok(())
//! }
//! # #[cfg(not(feature = "healthcare"))]
//! # fn main() {}
//! ```
//!
//! ## Authentication
//!
//! All Stedi APIs authenticate with an API key sent in the `Authorization` header as
//! `Key <api-key>`. [`StediClient`] wires this up for every service:
//!
//! ```no_run
//! use autogen_stedi::StediClient;
//!
//! let client = StediClient::new("your-api-key");
//! ```
//!
//! Each `client.<service>()` accessor returns the generated `Configuration` for that service. The
//! base URL (including the dated API version, e.g. `/2024-04-01`) is baked in from the spec; you can
//! still override `base_path` on the returned value to point at a proxy or mock server.
//!
//! ## Error handling
//!
//! Calls return `Result<T, apis::Error<E>>`, where `E` is the endpoint-specific error enum.
//! Each service module exposes its own `apis::Error`, which separates transport errors,
//! (de)serialization errors, and structured API error responses (carrying the HTTP status and body).
//!
//! ## Feature flags
//!
//! By default all services are enabled. To reduce compile time, select only what you need
//! (and a TLS backend — `native-tls` or `rustls`):
//!
//! ```toml
//! [dependencies]
//! autogen-stedi = { version = "0.1", default-features = false, features = ["healthcare", "native-tls"] }
//! ```

#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

#[cfg(feature = "claims")]
pub mod claims;
#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "enrollment")]
pub mod enrollment;
#[cfg(feature = "event-destinations")]
pub mod event_destinations;
#[cfg(feature = "healthcare")]
pub mod healthcare;
#[cfg(feature = "manager")]
pub mod manager;
#[cfg(feature = "payers")]
pub mod payers;

pub mod client;

pub use client::StediClient;
