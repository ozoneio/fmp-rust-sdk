//! Subscription configuration and advisory limits for Financial Modeling Prep.
//!
//! FMP does not document a public API that returns an API key's exact billing
//! plan. Supply the plan shown in the [account dashboard](https://site.financialmodelingprep.com/developer/docs/dashboard).
//! Successful endpoint access does not identify the plan or establish access to
//! other datasets. WebSocket access depends on the account's explicit cluster
//! entitlements; confirm them with FMP rather than deriving them from a plan.

use std::{error::Error, fmt, str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};

/// A configured subscription plan.
///
/// This value describes configuration, not a plan verified by the FMP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    /// FMP's Basic/free personal plan.
    #[serde(alias = "free")]
    Basic,
    /// FMP's Starter personal plan.
    Starter,
    /// FMP's Premium personal plan.
    Premium,
    /// FMP's Ultimate personal plan.
    Ultimate,
    /// The Build plan retained in FMP's published bandwidth policy.
    Build,
    /// A commercial Enterprise agreement with negotiated limits and access.
    Enterprise,
}

/// An advisory published request allowance, without additional account boosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimit {
    /// The allowed request count within the published window.
    pub requests: u32,
    /// The published request counting window.
    pub window: Duration,
}

impl Plan {
    /// Returns the standard request allowance published on 2026-10-10.
    ///
    /// These are advisory values from FMP's [pricing page](https://site.financialmodelingprep.com/pricing-plans).
    /// Add-ons and negotiated agreements can change the account's allowance.
    /// Build has no confirmed standard request allowance; Enterprise is custom.
    pub const fn published_rate_limit(self) -> Option<RateLimit> {
        match self {
            Self::Basic => Some(RateLimit {
                requests: 250,
                window: Duration::from_secs(86_400),
            }),
            Self::Starter => Some(RateLimit {
                requests: 300,
                window: Duration::from_secs(60),
            }),
            Self::Premium => Some(RateLimit {
                requests: 750,
                window: Duration::from_secs(60),
            }),
            Self::Ultimate => Some(RateLimit {
                requests: 3_000,
                window: Duration::from_secs(60),
            }),
            Self::Build | Self::Enterprise => None,
        }
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Basic => "basic",
            Self::Starter => "starter",
            Self::Premium => "premium",
            Self::Ultimate => "ultimate",
            Self::Build => "build",
            Self::Enterprise => "enterprise",
        })
    }
}

/// An unrecognized subscription plan name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanParseError {
    input: String,
}

impl PlanParseError {
    /// Returns the unrecognized input supplied to the parser.
    pub fn input(&self) -> &str {
        &self.input
    }
}

impl fmt::Display for PlanParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unrecognized FMP subscription plan {:?}; expected basic (or free), starter, premium, ultimate, build, or enterprise",
            self.input
        )
    }
}

impl Error for PlanParseError {}

impl FromStr for Plan {
    type Err = PlanParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "basic" | "free" => Ok(Self::Basic),
            "starter" => Ok(Self::Starter),
            "premium" => Ok(Self::Premium),
            "ultimate" => Ok(Self::Ultimate),
            "build" => Ok(Self::Build),
            "enterprise" => Ok(Self::Enterprise),
            _ => Err(PlanParseError {
                input: input.to_owned(),
            }),
        }
    }
}

/// Explicit account grants for FMP's three WebSocket clusters.
///
/// Grants are supplied by the account owner. FMP's [FAQ](https://site.financialmodelingprep.com/faqs)
/// describes cluster-specific entitlements and one simultaneous connection per
/// cluster. The server remains authoritative about authentication and access.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSocketEntitlements {
    /// Access to the stocks WebSocket cluster.
    pub stocks: bool,
    /// Access to the cryptocurrency WebSocket cluster.
    pub crypto: bool,
    /// Access to the forex WebSocket cluster.
    pub forex: bool,
}

impl WebSocketEntitlements {
    /// Creates a declaration with no WebSocket cluster grants.
    pub const fn none() -> Self {
        Self {
            stocks: false,
            crypto: false,
            forex: false,
        }
    }

    /// Creates a declaration with grants for all three WebSocket clusters.
    pub const fn all() -> Self {
        Self {
            stocks: true,
            crypto: true,
            forex: true,
        }
    }
}

/// Subscription details explicitly configured for an FMP account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    plan: Option<Plan>,
    websocket_entitlements: WebSocketEntitlements,
}

impl Subscription {
    /// Configures a known plan, or `None` when the billing plan is unknown.
    ///
    /// WebSocket entitlements are independent of the plan name and are supplied
    /// explicitly to support legacy accounts and negotiated agreements.
    pub const fn new(plan: Option<Plan>, websocket_entitlements: WebSocketEntitlements) -> Self {
        Self {
            plan,
            websocket_entitlements,
        }
    }

    /// Returns the configured billing plan, without claiming server verification.
    pub const fn plan(&self) -> Option<Plan> {
        self.plan
    }

    /// Returns the configured WebSocket cluster grants.
    pub const fn websocket_entitlements(&self) -> WebSocketEntitlements {
        self.websocket_entitlements
    }
}
