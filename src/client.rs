use std::{
    fmt,
    sync::Arc,
    time::{Duration, SystemTime},
};

use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue, RETRY_AFTER},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use url::Url;

use crate::{ApiErrorKind, Error, RequestContext, Result, Subscription, error::JsonDecodeError};

/// Bounded retries for transient transport errors, HTTP 408/429, and HTTP 5xx.
/// Authentication, access restrictions, and invalid schemas are never retried.
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    /// Maximum exponential delay and maximum acceptable server Retry-After.
    /// A larger Retry-After returns the provider error immediately.
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// HTTP transport configuration. Override the base URL for a gateway or a local
/// protocol test; unencrypted HTTP is accepted only on loopback hosts.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub base_url: Url,
    pub timeout: Duration,
    pub retry: RetryPolicy,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("https://financialmodelingprep.com/stable/")
                .expect("the built-in FMP URL is valid"),
            timeout: Duration::from_secs(30),
            retry: RetryPolicy::default(),
        }
    }
}

/// Reusable, asynchronous FMP connector. Clones share the connection pool.
/// API keys travel in a sensitive header, and redirects are rejected.
#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: Arc<str>,
    subscription: Subscription,
    config: ClientConfig,
}

impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Client")
            .field("api_key", &"[REDACTED]")
            .field("subscription", &self.subscription)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a client for the Stable API with a 30-second timeout and two
    /// transient-error retries. The supplied subscription is user configuration.
    pub fn new(api_key: &str, subscription: Subscription) -> Result<Self> {
        Self::with_config(api_key, subscription, ClientConfig::default())
    }

    /// Creates a client with explicit transport settings.
    pub fn with_config(
        api_key: &str,
        subscription: Subscription,
        config: ClientConfig,
    ) -> Result<Self> {
        validate_config(&config)?;
        if api_key.trim().is_empty() {
            return Err(configuration("the FMP API key must not be empty"));
        }
        let mut key = HeaderValue::from_str(api_key).map_err(|_| {
            configuration("the FMP API key contains invalid HTTP header characters")
        })?;
        key.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert("apikey", key);
        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|error| {
                configuration(&format!(
                    "cannot initialize HTTP transport: {}",
                    error.without_url()
                ))
            })?;
        Ok(Self {
            http,
            api_key: Arc::from(api_key),
            subscription,
            config,
        })
    }

    /// Returns the configured plan and cluster grants. They are not server-verified.
    pub const fn subscription(&self) -> &Subscription {
        &self.subscription
    }

    #[cfg(feature = "websocket")]
    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Requests any Stable JSON endpoint and validates its response against `T`.
    /// Define a `Deserialize` model for endpoints without a named SDK wrapper.
    /// Paths must be relative (e.g. `"quote"`); serialize query parameters through
    /// `params`, never append them to the path. An API error inside HTTP 200 is
    /// returned as an error even if `T` could deserialize that object.
    pub async fn get<T, Q>(&self, path: &str, params: &Q) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let response = self.execute(path, params).await?;
        serde_json::from_slice(&response.body).map_err(|source| Error::Decode {
            request: response.request,
            status: response.status,
            body: self.redact(&String::from_utf8_lossy(&response.body)),
            source: JsonDecodeError {
                message: self.redact(&source.to_string()),
                line: source.line(),
                column: source.column(),
            },
        })
    }

    /// Requests a text endpoint, including FMP CSV bulk datasets. Provider errors
    /// are still checked before returning the UTF-8 text.
    pub async fn get_text<Q>(&self, path: &str, params: &Q) -> Result<String>
    where
        Q: Serialize + ?Sized,
    {
        let response = self.execute(path, params).await?;
        String::from_utf8(response.body).map_err(|source| Error::Encoding {
            request: response.request,
            status: response.status,
            source: source.utf8_error(),
        })
    }

    /// Requests a binary endpoint, such as financial reports in XLSX format.
    pub async fn get_bytes<Q>(&self, path: &str, params: &Q) -> Result<Vec<u8>>
    where
        Q: Serialize + ?Sized,
    {
        Ok(self.execute(path, params).await?.body)
    }

    async fn execute<Q>(&self, path: &str, params: &Q) -> Result<ResponseData>
    where
        Q: Serialize + ?Sized,
    {
        validate_path(path)?;
        let url =
            self.config.base_url.join(path).map_err(|error| {
                configuration(&format!("invalid endpoint path {path:?}: {error}"))
            })?;
        let context = RequestContext {
            url: self.redact(url.as_str()),
        };
        let request =
            self.http
                .get(url)
                .query(params)
                .build()
                .map_err(|source| Error::Request {
                    request: context,
                    source: source.without_url(),
                })?;
        if request
            .url()
            .query_pairs()
            .any(|(name, _)| is_credential_param(&name))
        {
            return Err(configuration(
                "query parameters must not contain an API key or authorization; the client's API key is sent in a header",
            ));
        }
        let context = RequestContext {
            url: self.redact(request.url().as_str()),
        };
        for attempt in 0..=self.config.retry.max_retries {
            let outgoing = request
                .try_clone()
                .ok_or_else(|| configuration("the GET request cannot be cloned for retry"))?;
            let result = self.send(outgoing, &context).await;
            match result {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let delay = retry_delay(&error, attempt, &self.config.retry);
                    if let Some(delay) = delay {
                        tracing::warn!(request = %context, attempt = attempt + 1, delay_ms = delay.as_millis(), error = %error, "FMP request failed; retrying");
                        tokio::time::sleep(delay).await;
                    } else {
                        return Err(error);
                    }
                }
            }
        }
        unreachable!("the final retry always returns its result")
    }

    async fn send(
        &self,
        request: reqwest::Request,
        context: &RequestContext,
    ) -> Result<ResponseData> {
        let response = self
            .http
            .execute(request)
            .await
            .map_err(|source| Error::Transport {
                request: context.clone(),
                source: source.without_url(),
            })?;
        let status = response.status();
        let retry_after = parse_retry_after(response.headers(), SystemTime::now());
        let body = response
            .bytes()
            .await
            .map_err(|source| Error::ResponseBody {
                request: context.clone(),
                status: status.as_u16(),
                source: source.without_url(),
            })?
            .to_vec();
        let retry_after = retry_after.map_err(|reason| Error::ResponseHeader {
            request: context.clone(),
            status: status.as_u16(),
            header: "Retry-After",
            reason: self.redact(&reason),
            body: self.redact(&String::from_utf8_lossy(&body)),
        })?;
        if !status.is_success() {
            return Err(self.api_failure(
                context,
                status,
                &body,
                &String::from_utf8_lossy(&body),
                retry_after,
            ));
        }
        let provider_error = parse_provider_error(&body).map_err(|source| Error::Decode {
            request: context.clone(),
            status: status.as_u16(),
            body: self.redact(&String::from_utf8_lossy(&body)),
            source: JsonDecodeError {
                message: self.redact(&source.to_string()),
                line: source.line(),
                column: source.column(),
            },
        })?;
        if let Some(provider_error) = provider_error {
            return Err(self.api_failure(
                context,
                status,
                &body,
                &provider_error.message,
                retry_after,
            ));
        }
        Ok(ResponseData {
            request: context.clone(),
            status: status.as_u16(),
            body,
        })
    }

    fn redact(&self, value: &str) -> String {
        crate::credentials::redact(value, &self.api_key)
    }

    fn api_failure(
        &self,
        request: &RequestContext,
        status: StatusCode,
        body: &[u8],
        message: &str,
        retry_after: Option<Duration>,
    ) -> Error {
        Error::Api {
            request: request.clone(),
            kind: classify_error(status, message),
            status: status.as_u16(),
            body: self.redact(&String::from_utf8_lossy(body)),
            retry_after,
        }
    }
}

struct ResponseData {
    request: RequestContext,
    status: u16,
    body: Vec<u8>,
}

#[derive(Deserialize)]
struct ProviderError {
    #[serde(rename = "Error Message", alias = "error", alias = "Error")]
    message: String,
}

#[derive(Deserialize)]
struct ErrorEnvelopeMarker {
    #[serde(
        rename = "Error Message",
        alias = "error",
        alias = "Error",
        default,
        deserialize_with = "present_error_field"
    )]
    present: bool,
}

fn present_error_field<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Only the field's presence is inspected here; the next pass validates its
    // required String schema and reports null/numeric/malformed errors explicitly.
    serde::de::IgnoredAny::deserialize(deserializer)?;
    Ok(true)
}

fn parse_provider_error(
    body: &[u8],
) -> std::result::Result<Option<ProviderError>, serde_json::Error> {
    if body.iter().find(|byte| !byte.is_ascii_whitespace()) != Some(&b'{') {
        return Ok(None);
    }
    let marker: ErrorEnvelopeMarker = serde_json::from_slice(body)?;
    if marker.present {
        serde_json::from_slice(body).map(Some)
    } else {
        Ok(None)
    }
}

fn configuration(reason: &str) -> Error {
    Error::Configuration {
        reason: reason.to_owned(),
    }
}

fn validate_config(config: &ClientConfig) -> Result<()> {
    let url = &config.base_url;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err(configuration(
            "base_url must use HTTPS, or HTTP on a loopback host",
        ));
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.path().ends_with('/')
    {
        return Err(configuration(
            "base_url must end with '/' and must not contain credentials, query parameters, or a fragment",
        ));
    }
    if config.timeout.is_zero() {
        return Err(configuration("request timeout must be greater than zero"));
    }
    if config.retry.max_retries > 10 {
        return Err(configuration("max_retries must be at most 10"));
    }
    if config.retry.max_retries > 0
        && (config.retry.initial_delay.is_zero()
            || config.retry.max_delay < config.retry.initial_delay)
    {
        return Err(configuration(
            "retry delays must be nonzero and max_delay must be at least initial_delay",
        ));
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || !path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_'))
    {
        return Err(configuration(
            "endpoint path must contain only relative nonempty alphanumeric, '-', or '_' segments; put query parameters in params",
        ));
    }
    Ok(())
}

fn is_credential_param(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase().replace(['_', '-'], "");
    matches!(
        normalized.as_str(),
        "apikey" | "authorization" | "accesskey" | "accesstoken"
    )
}

fn classify_error(status: StatusCode, message: &str) -> ApiErrorKind {
    let lower = message.to_ascii_lowercase();
    if status == StatusCode::UNAUTHORIZED
        || lower.contains("invalid api key")
        || lower.contains("invalid apikey")
    {
        ApiErrorKind::Authentication
    } else if status == StatusCode::TOO_MANY_REQUESTS
        || lower.contains("limit reach")
        || lower.contains("rate limit")
        || lower.contains("bandwidth limit")
    {
        ApiErrorKind::RateLimited
    } else if lower.contains("premium query")
        || lower.contains("restricted endpoint")
        || lower.contains("upgrade your")
        || lower.contains("subscription required")
        || lower.contains("not available under your current subscription")
        || lower.contains("not available for your subscription")
    {
        ApiErrorKind::PlanRestricted
    } else if !status.is_success() {
        ApiErrorKind::HttpStatus
    } else {
        ApiErrorKind::Provider
    }
}

fn parse_retry_after(
    headers: &HeaderMap,
    now: SystemTime,
) -> std::result::Result<Option<Duration>, String> {
    let Some(header) = headers.get(RETRY_AFTER) else {
        return Ok(None);
    };
    let text = header
        .to_str()
        .map_err(|error| format!("invalid header encoding: {error}"))?;
    if let Ok(seconds) = text.parse::<u64>() {
        return Ok(Some(Duration::from_secs(seconds)));
    }
    let date = httpdate::parse_http_date(text)
        .map_err(|error| format!("invalid value {text:?}: {error}"))?;
    Ok(Some(date.duration_since(now).unwrap_or(Duration::ZERO)))
}

fn retry_delay(error: &Error, attempt: u32, policy: &RetryPolicy) -> Option<Duration> {
    if attempt >= policy.max_retries {
        return None;
    }
    let retry_after = match error {
        Error::Api {
            kind: ApiErrorKind::Authentication | ApiErrorKind::PlanRestricted,
            ..
        } => return None,
        Error::Transport { source, .. } if source.is_connect() || source.is_timeout() => None,
        Error::ResponseBody { .. } => None,
        Error::Api {
            status,
            kind,
            retry_after,
            ..
        } if *status == 408
            || *kind == ApiErrorKind::RateLimited
            || (500..=599).contains(status) =>
        {
            *retry_after
        }
        _ => return None,
    };
    match retry_after {
        Some(delay) if delay <= policy.max_delay => Some(delay),
        Some(_) => None,
        None => Some(
            policy
                .initial_delay
                .saturating_mul(2_u32.saturating_pow(attempt))
                .min(policy.max_delay),
        ),
    }
}
