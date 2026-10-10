use serde::Serialize;

use crate::{ApiErrorKind, Client, Error, Plan, Result};

/// Observed access to one endpoint and parameter combination. An empty response
/// still counts as allowed, and says nothing about other symbols or date ranges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessOutcome {
    Allowed,
    /// FMP explicitly reported a subscription restriction. HTTP 403 alone is
    /// insufficient evidence; other forbidden responses remain errors.
    Restricted {
        status: u16,
        body: String,
    },
}

/// Subscription configuration alongside one server-observed capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessReport {
    /// The account owner's declaration; not an automatically detected plan.
    pub configured_plan: Option<Plan>,
    pub endpoint: String,
    pub outcome: AccessOutcome,
}

impl Client {
    /// Verifies whether the key can call an endpoint with these parameters.
    /// This costs a real API request. Invalid keys, rate limits, outages, and
    /// unclassified forbidden responses propagate as errors; they never become
    /// evidence of a lower plan. FMP does not expose a documented billing-plan
    /// lookup, so the report keeps the configured plan separate from the result.
    /// Successful bodies are not checked against a dataset schema; use a typed
    /// endpoint method or `get` when the data itself must also be validated.
    pub async fn probe_access<Q>(&self, path: &str, params: &Q) -> Result<AccessReport>
    where
        Q: Serialize + ?Sized,
    {
        let outcome = match self.get_bytes(path, params).await {
            Ok(_) => AccessOutcome::Allowed,
            Err(Error::Api {
                kind: ApiErrorKind::PlanRestricted,
                status,
                body,
                ..
            }) => AccessOutcome::Restricted { status, body },
            Err(error) => return Err(error),
        };
        Ok(AccessReport {
            configured_plan: self.subscription().plan(),
            endpoint: path.to_owned(),
            outcome,
        })
    }
}
