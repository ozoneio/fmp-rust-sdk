use std::time::Duration;

/// A result from an FMP HTTP request or configuration operation.
pub type Result<T> = std::result::Result<T, Error>;

/// The request URL and query parameters. Authentication is sent in a header,
/// never included in this value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestContext {
    pub url: String,
}

impl std::fmt::Display for RequestContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.url)
    }
}

/// Classification of a provider error; the original status and body are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApiErrorKind {
    Authentication,
    PlanRestricted,
    RateLimited,
    HttpStatus,
    Provider,
}

/// A JSON/schema error whose diagnostic message has had credentials removed.
/// The original serde error cannot be retained because it may quote a secret
/// response value in its Display, Debug, or source chain.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct JsonDecodeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Specific, contextual failures. API keys are redacted from all retained bodies.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid SDK configuration: {reason}")]
    Configuration { reason: String },
    #[error("cannot construct GET {request}: {source}")]
    Request {
        request: RequestContext,
        #[source]
        source: reqwest::Error,
    },
    #[error("GET {request} failed in transport: {source}")]
    Transport {
        request: RequestContext,
        #[source]
        source: reqwest::Error,
    },
    #[error("GET {request} returned HTTP {status}, but its body could not be read: {source}")]
    ResponseBody {
        request: RequestContext,
        status: u16,
        #[source]
        source: reqwest::Error,
    },
    #[error("GET {request} failed ({kind:?}, HTTP {status}): {body}")]
    Api {
        request: RequestContext,
        kind: ApiErrorKind,
        status: u16,
        body: String,
        retry_after: Option<Duration>,
    },
    #[error("GET {request} returned invalid JSON/schema (HTTP {status}): {source}; body: {body}")]
    Decode {
        request: RequestContext,
        status: u16,
        body: String,
        #[source]
        source: JsonDecodeError,
    },
    #[error("GET {request} returned non-UTF-8 text (HTTP {status}): {source}")]
    Encoding {
        request: RequestContext,
        status: u16,
        #[source]
        source: std::str::Utf8Error,
    },
    #[error(
        "GET {request} returned an invalid {header} header (HTTP {status}): {reason}; body: {body}"
    )]
    ResponseHeader {
        request: RequestContext,
        status: u16,
        header: &'static str,
        reason: String,
        body: String,
    },
}
