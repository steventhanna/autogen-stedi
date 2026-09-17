//! Hand-written entry point that adds authentication on top of the generated per-service
//! configurations. This is the only file (besides `lib.rs`) that is not generated.

use std::sync::Arc;

/// Holds your Stedi API key and hands out a ready-to-use [`Configuration`] for each API.
///
/// Stedi exposes several independent APIs, each on its own host. `StediClient` stores the key once
/// and produces a per-service `Configuration` — with the correct base URL and auth already set —
/// via accessors like [`healthcare`](StediClient::healthcare) and [`claims`](StediClient::claims).
///
/// ```no_run
/// use autogen_stedi::StediClient;
/// let client = StediClient::new("your-api-key");
/// # #[cfg(feature = "healthcare")]
/// let healthcare = client.healthcare(); // pass to autogen_stedi::healthcare::apis functions
/// ```
///
/// Use [`StediClient::builder`] to attach `reqwest_middleware` middleware, such as a tracing or
/// retry layer, to every request the client makes.
#[derive(Debug, Clone)]
pub struct StediClient {
    api_key: String,
    http: reqwest_middleware::ClientWithMiddleware,
}

impl StediClient {
    /// Create a client from a Stedi API key, with no middleware attached.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder(api_key).build()
    }

    /// Start building a [`StediClient`] with middleware attached to its HTTP client.
    pub fn builder(api_key: impl Into<String>) -> StediClientBuilder {
        StediClientBuilder {
            api_key: api_key.into(),
            middleware: Vec::new(),
        }
    }
}

/// Builds a [`StediClient`] with a chain of `reqwest_middleware` middleware.
///
/// Middleware runs in the order it is added, wrapping every HTTP request the resulting client
/// makes across all Stedi services.
pub struct StediClientBuilder {
    api_key: String,
    middleware: Vec<Arc<dyn reqwest_middleware::Middleware>>,
}

impl StediClientBuilder {
    /// Append a middleware to the chain.
    pub fn with<M: reqwest_middleware::Middleware>(mut self, middleware: M) -> Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    /// Append a shared middleware to the chain.
    pub fn with_arc(mut self, middleware: Arc<dyn reqwest_middleware::Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Build the [`StediClient`].
    pub fn build(self) -> StediClient {
        let mut builder = reqwest_middleware::ClientBuilder::new(reqwest::Client::new());
        for middleware in self.middleware {
            builder = builder.with_arc(middleware);
        }
        StediClient {
            api_key: self.api_key,
            http: builder.build(),
        }
    }
}

impl std::fmt::Debug for StediClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StediClientBuilder")
            .field("middleware_count", &self.middleware.len())
            .finish()
    }
}

/// Generates a `StediClient::<method>()` accessor returning the named service's `Configuration`,
/// with the API key wired into the `Authorization: Key <api-key>` header, the client's middleware
/// chain, and a crate user-agent. The base URL is taken from the generated `Configuration::default()`
/// (i.e. the spec's server).
macro_rules! service_config {
    ($method:ident, $feature:literal, $module:ident) => {
        #[cfg(feature = $feature)]
        impl StediClient {
            #[doc = concat!("Configuration for the Stedi `", stringify!($module), "` API.")]
            pub fn $method(&self) -> crate::$module::apis::configuration::Configuration {
                let mut config = crate::$module::apis::configuration::Configuration::default();
                config.client = self.http.clone();
                config.api_key = Some(crate::$module::apis::configuration::ApiKey {
                    prefix: Some("Key".to_string()),
                    key: self.api_key.clone(),
                });
                config.user_agent =
                    Some(format!("autogen-stedi/{}", env!("CARGO_PKG_VERSION")));
                config
            }
        }
    };
}

service_config!(claims, "claims", claims);
service_config!(core, "core", core);
service_config!(enrollment, "enrollment", enrollment);
service_config!(event_destinations, "event-destinations", event_destinations);
service_config!(healthcare, "healthcare", healthcare);
service_config!(manager, "manager", manager);
service_config!(payers, "payers", payers);
