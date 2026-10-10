//! Typed responses from the FMP Stable API.
//!
//! Identifying fields are required, dates are parsed, and unrelated response
//! fields are ignored. Financial values use `f64` because FMP returns both integer
//! amounts and fractional values; use your own decimal model with `Client::get`
//! when exact decimal arithmetic is required.

mod company;
mod events;
mod financials;
mod growth;
mod market;
mod metrics;
mod prices;
mod quotes;
mod ttm;

pub use company::*;
pub use events::*;
pub use financials::*;
pub use growth::*;
pub use market::*;
pub use metrics::*;
pub use prices::*;
pub use quotes::*;
pub use ttm::*;

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serializer, de::Error};

fn non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        return Err(D::Error::custom("required identifier must not be empty"));
    }
    Ok(value)
}

/// FMP represents unknown corporate-action dates with either null or an empty string.
fn optional_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        None => Ok(None),
        Some(value) if value.is_empty() => Ok(None),
        Some(value) => NaiveDate::parse_from_str(&value, "%Y-%m-%d")
            .map(Some)
            .map_err(D::Error::custom),
    }
}

/// Provider-local timestamps contain no UTC offset; do not interpret them as UTC.
mod timestamp {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S").map_err(D::Error::custom)
    }

    pub fn serialize<S>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}

mod optional_timestamp {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<String>::deserialize(deserializer)?
            .map(|value| {
                NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S").map_err(D::Error::custom)
            })
            .transpose()
    }

    pub fn serialize<S>(value: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => {
                serializer.serialize_some(&value.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            None => serializer.serialize_none(),
        }
    }
}
