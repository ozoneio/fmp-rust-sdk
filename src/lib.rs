//! An asynchronous SDK for the [Financial Modeling Prep Stable API](https://site.financialmodelingprep.com/developer/docs).
//!
//! ```no_run
//! use fmp_rust_sdk::{Client, WebSocketEntitlements};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::from_env(WebSocketEntitlements::none())?;
//! let quotes = client.quote("AAPL").await?;
//! println!("{:?}", quotes);
//! # Ok(())
//! # }
//! ```
//!
//! FMP does not document an API-key endpoint that returns the billing plan. Supply
//! the plan shown in the FMP dashboard through [`Subscription`]; [`Client::probe_access`]
//! verifies access to a specific request without claiming to identify an exact tier.
//! [`Client::from_env`] and [`Client::from_dotenv`] read optional `FMP_PLAN`,
//! defaulting to the free plan ([`Plan::Basic`]) when it is unset or blank.
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
