//! Hand-written entry point that adds authentication on top of the generated per-service
//! configurations. This is the only file (besides `lib.rs`) that is not generated.

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
#[derive(Debug, Clone)]
pub struct StediClient {
    api_key: String,
}

impl StediClient {
    /// Create a client from a Stedi API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

/// Generates a `StediClient::<method>()` accessor returning the named service's `Configuration`,
/// with the API key wired into the `Authorization: Key <api-key>` header and a crate user-agent.
/// The base URL is taken from the generated `Configuration::default()` (i.e. the spec's server).
macro_rules! service_config {
    ($method:ident, $feature:literal, $module:ident) => {
        #[cfg(feature = $feature)]
        impl StediClient {
            #[doc = concat!("Configuration for the Stedi `", stringify!($module), "` API.")]
            pub fn $method(&self) -> crate::$module::apis::configuration::Configuration {
                let mut config = crate::$module::apis::configuration::Configuration::default();
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
