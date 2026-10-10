//! An asynchronous SDK for the [Financial Modeling Prep Stable API](https://site.financialmodelingprep.com/developer/docs).
//!
//! ```no_run
//! use fmp_rust_sdk::{Client, Subscription, WebSocketEntitlements};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let key = std::env::var("FMP_API_KEY")?;
//! let subscription = Subscription::new(None, WebSocketEntitlements::default());
//! let client = Client::new(&key, subscription)?;
//! let quotes = client.quote("AAPL").await?;
//! println!("{:?}", quotes);
//! # Ok(())
//! # }
//! ```
//!
//! FMP does not document an API-key endpoint that returns the billing plan. Supply
//! the plan shown in the FMP dashboard through [`Subscription`]; [`Client::probe_access`]
//! verifies access to a specific request without claiming to identify an exact tier.
//! WebSocket grants are independent of the configured plan and must be explicitly
//! supplied after FMP approves the relevant clusters.

mod access;
mod client;
mod config;
mod credentials;
pub mod endpoints;
mod error;
pub mod models;
pub mod plan;
#[cfg(feature = "websocket")]
pub mod websocket;

pub use access::{AccessOutcome, AccessReport};
pub use client::{Client, ClientConfig, RetryPolicy};
pub use error::{ApiErrorKind, Error, JsonDecodeError, RequestContext, Result};
pub use plan::{Plan, Subscription, WebSocketEntitlements};
