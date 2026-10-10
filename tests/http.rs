use std::time::Duration;

use chrono::NaiveDate;
use fmp_rust_sdk::{
    AccessOutcome, ApiErrorKind, Client, ClientConfig, Error, Plan, RetryPolicy, Subscription,
    WebSocketEntitlements,
    endpoints::{DateRange, Period, StatementOptions},
};
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
    time::timeout,
};
use url::Url;

const API_KEY: &str = "sdk key+\"/secret";
const SERVER_TIMEOUT: Duration = Duration::from_secs(5);

struct Server {
    base_url: Url,
    requests: JoinHandle<Vec<String>>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Quote {
    symbol: String,
    price: f64,
}

#[derive(Debug, Deserialize)]
struct ProviderMessage {
    #[serde(rename = "Error Message")]
    _message: String,
}

fn response(status: &str, headers: &[(&str, &str)], body: &str) -> String {
    let extra_headers: String = headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect();
    format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n{extra_headers}\r\n{body}",
        body.len()
    )
}

async fn start_server(responses: Vec<String>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the integration HTTP server");
    let address = listener.local_addr().expect("read the HTTP server address");
    let requests = tokio::spawn(async move {
        let mut requests = Vec::with_capacity(responses.len());
        for response in responses {
            let (mut stream, _) = timeout(SERVER_TIMEOUT, listener.accept())
                .await
                .expect("the SDK must contact the HTTP server before the deadline")
                .expect("accept an SDK request");
            let request = timeout(SERVER_TIMEOUT, async {
                let mut bytes = Vec::new();
                while !bytes.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                    let count = stream.read_buf(&mut bytes).await.expect("read SDK headers");
                    assert!(
                        count > 0,
                        "the SDK closed the request before sending headers"
                    );
                    assert!(
                        bytes.len() < 65_536,
                        "request headers exceeded the test limit"
                    );
                }
                String::from_utf8(bytes).expect("SDK request headers must be UTF-8")
            })
            .await
            .expect("the SDK must finish sending HTTP headers before the deadline");
            requests.push(request);
            timeout(SERVER_TIMEOUT, async {
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("send the integration HTTP response");
                stream.shutdown().await.expect("close the HTTP response");
            })
            .await
            .expect("the integration HTTP response must finish before the deadline");
        }
        requests
    });
    Server {
        base_url: Url::parse(&format!("http://{address}/stable/"))
            .expect("the loopback URL is valid"),
        requests,
    }
}

async fn finish(server: Server) -> Vec<String> {
    timeout(SERVER_TIMEOUT, server.requests)
        .await
        .expect("the integration HTTP server must complete before the deadline")
        .expect("the integration HTTP server must complete successfully")
}

fn retry_policy(max_retries: u32) -> RetryPolicy {
    RetryPolicy {
        max_retries,
        initial_delay: Duration::from_millis(5),
        max_delay: Duration::from_millis(20),
    }
}

fn make_client(base_url: &Url, retry: RetryPolicy) -> Client {
    Client::with_config(
        API_KEY,
        Subscription::new(Some(Plan::Premium), WebSocketEntitlements::none()),
        ClientConfig {
            base_url: base_url.clone(),
            timeout: Duration::from_secs(2),
            retry,
        },
    )
    .expect("valid integration client configuration")
}

fn request_url(request: &str) -> Url {
    let target = request
        .lines()
        .next()
        .expect("the SDK request must contain a request line")
        .split_whitespace()
        .nth(1)
        .expect("the SDK request must contain a request target");
    Url::parse(&format!("http://localhost{target}")).expect("the SDK request target is valid")
}

#[tokio::test]
async fn typed_sdk_workflow_decodes_market_company_financial_and_calendar_data() {
    let server = start_server(vec![
        response("200 OK", &[], r#"[{"symbol":"AAPL","price":123.5}]"#),
        response("200 OK", &[], r#"[{"symbol":"AAPL","companyName":"Apple Inc.","ipoDate":"1980-12-12"}]"#),
        response("200 OK", &[], r#"[{"date":"2025-09-27","symbol":"AAPL","reportedCurrency":"USD","cik":"0000320193","filingDate":"2025-10-31","acceptedDate":"2025-10-31 06:01:26","fiscalYear":"2025","period":"FY","revenue":416161000000,"netIncome":112010000000}]"#),
        response("200 OK", &[], r#"[{"symbol":"AAPL","date":"2025-12-31","open":273.06,"high":273.68,"low":271.75,"close":271.86,"volume":27293639,"change":-1.2,"changePercent":-0.43946,"vwap":272.5875}]"#),
        response("200 OK", &[], r#"[{"symbol":"AAPL","date":"2025-12-31","epsEstimated":1.2,"lastUpdated":""}]"#),
        response("200 OK", &[], r#"[{"price":1.0}]"#),
        response("200 OK", &[], r#"[{"symbol":"  ","price":1.0}]"#),
    ]).await;
    let client = make_client(&server.base_url, retry_policy(0));
    let from = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let to = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let dates = DateRange {
        from: Some(from),
        to: Some(to),
    };

    let quotes = client.quote("AAPL").await.unwrap();
    assert_eq!(quotes[0].symbol, "AAPL");
    assert_eq!(quotes[0].price, 123.5);
    let profiles = client.profile("AAPL").await.unwrap();
    assert_eq!(profiles[0].company_name, "Apple Inc.");
    assert_eq!(profiles[0].ipo_date, NaiveDate::from_ymd_opt(1980, 12, 12));
    let statements = client
        .income_statement(
            "AAPL",
            StatementOptions {
                period: Period::Annual,
                limit: 5,
            },
        )
        .await
        .unwrap();
    let metadata = &statements[0].metadata;
    assert_eq!(metadata.symbol, "AAPL");
    assert_eq!(metadata.date, NaiveDate::from_ymd_opt(2025, 9, 27).unwrap());
    assert_eq!(metadata.period, "FY");
    assert_eq!(metadata.reported_currency, "USD");
    assert_eq!(
        metadata.accepted_date,
        NaiveDate::from_ymd_opt(2025, 10, 31)
            .unwrap()
            .and_hms_opt(6, 1, 26)
    );
    assert_eq!(statements[0].revenue, Some(416161000000.0));
    let prices = client.historical_prices("AAPL", dates).await.unwrap();
    assert_eq!(prices[0].symbol, "AAPL");
    assert_eq!(prices[0].date, to);
    assert_eq!(prices[0].close, 271.86);
    let earnings = client.earnings_calendar(dates).await.unwrap();
    assert_eq!(earnings[0].symbol, "AAPL");
    assert_eq!(earnings[0].date, to);
    assert_eq!(earnings[0].last_updated, None);
    for _ in 0..2 {
        assert!(matches!(
            client.quote("AAPL").await,
            Err(Error::Decode { .. })
        ));
    }

    let requests = finish(server).await;
    let expected = [
        ("/stable/quote", vec![("symbol", "AAPL")]),
        ("/stable/profile", vec![("symbol", "AAPL")]),
        (
            "/stable/income-statement",
            vec![("symbol", "AAPL"), ("period", "annual"), ("limit", "5")],
        ),
        (
            "/stable/historical-price-eod/full",
            vec![
                ("symbol", "AAPL"),
                ("from", "2025-01-01"),
                ("to", "2025-12-31"),
            ],
        ),
        (
            "/stable/earnings-calendar",
            vec![("from", "2025-01-01"), ("to", "2025-12-31")],
        ),
        ("/stable/quote", vec![("symbol", "AAPL")]),
        ("/stable/quote", vec![("symbol", "AAPL")]),
    ];
    assert_eq!(requests.len(), expected.len());
    for (request, (path, expected_query)) in requests.iter().zip(expected) {
        let url = request_url(request);
        assert_eq!(url.path(), path);
        let mut query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        let mut expected_query: Vec<(String, String)> = expected_query
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect();
        query.sort_unstable();
        expected_query.sort_unstable();
        assert_eq!(query, expected_query);
    }
}

#[tokio::test]
async fn authenticated_requests_encode_query_and_validate_required_response_fields() {
    let server = start_server(vec![
        response(
            "200 OK",
            &[],
            r#"[{"symbol":"AAPL","price":123.5,"extra":"ignored"}]"#,
        ),
        response("200 OK", &[], r#"[{"symbol":"AAPL"}]"#),
        response("200 OK", &[], "not JSON"),
    ])
    .await;
    let client = make_client(&server.base_url, retry_policy(0));
    let params = [("query", "AAPL & B+S/東京")];
    let quotes: Vec<Quote> = client.get("search-symbol", &params).await.unwrap();
    assert_eq!(
        quotes,
        vec![Quote {
            symbol: "AAPL".to_owned(),
            price: 123.5,
        }]
    );
    for expected_body in [r#"[{"symbol":"AAPL"}]"#, "not JSON"] {
        let error = client
            .get::<Vec<Quote>, _>("quote", &[("symbol", "AAPL")])
            .await
            .unwrap_err();
        match error {
            Error::Decode {
                status,
                body,
                request,
                ..
            } => {
                assert_eq!(status, 200);
                assert_eq!(body, expected_body);
                assert!(request.url.contains("symbol=AAPL"));
            }
            error => panic!("expected a contextual schema error, got {error:?}"),
        }
    }
    let requests = finish(server).await;
    assert_eq!(requests.len(), 3);
    let request = &requests[0];
    assert!(
        request
            .lines()
            .any(|line| line == format!("apikey: {API_KEY}"))
    );
    let target_url = request_url(request);
    assert_eq!(target_url.path(), "/stable/search-symbol");
    let query: Vec<(String, String)> = target_url.query_pairs().into_owned().collect();
    assert_eq!(
        query,
        vec![("query".to_owned(), "AAPL & B+S/東京".to_owned())]
    );
    assert!(!target_url.as_str().contains(API_KEY));
}

#[tokio::test]
async fn provider_errors_and_access_probes_preserve_authentication_and_restriction_evidence() {
    let server = start_server(vec![
        response(
            "200 OK",
            &[],
            r#"{"Error Message":"Premium Query: subscription required"}"#,
        ),
        response("200 OK", &[], "[]"),
        response(
            "403 Forbidden",
            &[],
            r#"{"Error Message":"Restricted endpoint: upgrade your plan"}"#,
        ),
        response(
            "401 Unauthorized",
            &[],
            r#"{"Error Message":"Invalid API KEY"}"#,
        ),
        response("403 Forbidden", &[], "Forbidden by gateway"),
        response(
            "503 Service Unavailable",
            &[],
            r#"{"Error Message":"Restricted endpoint: subscription required"}"#,
        ),
        response("200 OK", &[], r#"{"Error Message":null}"#),
    ])
    .await;
    let client = make_client(&server.base_url, retry_policy(2));
    let params = [("symbol", "AAPL")];
    let error = client
        .get::<ProviderMessage, _>("quote", &params)
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Api {
            status: 200,
            kind: ApiErrorKind::PlanRestricted,
            ..
        }
    ));
    let report = client.probe_access("quote", &params).await.unwrap();
    assert_eq!(report.configured_plan, Some(Plan::Premium));
    assert_eq!(report.outcome, AccessOutcome::Allowed);
    let report = client.probe_access("quote", &params).await.unwrap();
    assert_eq!(report.configured_plan, Some(Plan::Premium));
    assert!(matches!(
        report.outcome,
        AccessOutcome::Restricted { status: 403, .. }
    ));
    let error = client.probe_access("quote", &params).await.unwrap_err();
    assert!(matches!(
        error,
        Error::Api {
            status: 401,
            kind: ApiErrorKind::Authentication,
            ..
        }
    ));
    let error = client.probe_access("quote", &params).await.unwrap_err();
    assert!(matches!(
        error,
        Error::Api {
            status: 403,
            kind: ApiErrorKind::HttpStatus,
            ..
        }
    ));
    let report = client.probe_access("quote", &params).await.unwrap();
    assert!(matches!(
        report.outcome,
        AccessOutcome::Restricted { status: 503, .. }
    ));
    client
        .probe_access("quote", &params)
        .await
        .expect_err("a malformed provider error envelope cannot establish allowed access");
    assert_eq!(finish(server).await.len(), 7);
}

#[tokio::test]
async fn transient_requests_retry_with_server_delay_and_return_the_last_failure() {
    let server = start_server(vec![
        response("503 Service Unavailable", &[], "{temporarily unavailable"),
        response(
            "429 Too Many Requests",
            &[("Retry-After", "0")],
            "rate limit",
        ),
        response("200 OK", &[], r#"{"symbol":"AAPL","price":1.0}"#),
    ])
    .await;
    let client = make_client(&server.base_url, retry_policy(2));
    let quote: Quote = client.get("quote", &[("symbol", "AAPL")]).await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
    assert_eq!(finish(server).await.len(), 3);

    let server = start_server(vec![
        response("503 Service Unavailable", &[], "first failure"),
        response(
            "429 Too Many Requests",
            &[("Retry-After", "0")],
            "last rate limit failure",
        ),
    ])
    .await;
    let client = make_client(&server.base_url, retry_policy(1));
    let error = client
        .get_bytes("quote", &[("symbol", "AAPL")])
        .await
        .unwrap_err();
    match error {
        Error::Api {
            status,
            kind,
            body,
            retry_after,
            ..
        } => {
            assert_eq!(status, 429);
            assert_eq!(kind, ApiErrorKind::RateLimited);
            assert_eq!(body, "last rate limit failure");
            assert_eq!(retry_after, Some(Duration::ZERO));
        }
        error => panic!("expected the last provider error, got {error:?}"),
    }
    assert_eq!(finish(server).await.len(), 2);

    let server = start_server(vec![response(
        "429 Too Many Requests",
        &[("Retry-After", "120")],
        "rate limit requiring a long pause",
    )])
    .await;
    let client = make_client(&server.base_url, retry_policy(2));
    let error = client
        .get_bytes("quote", &[("symbol", "AAPL")])
        .await
        .unwrap_err();
    assert!(
        matches!(error, Error::Api { retry_after: Some(delay), .. } if delay == Duration::from_secs(120))
    );
    assert_eq!(finish(server).await.len(), 1);
}

#[tokio::test]
async fn provider_and_schema_diagnostics_redact_echoed_credentials() {
    let body = serde_json::json!({"Error Message": format!("rejected {API_KEY}")}).to_string();
    let invalid_quote = serde_json::json!({"symbol":"AAPL", "price":API_KEY}).to_string();
    let server = start_server(vec![
        response("400 Bad Request", &[], &body),
        response("200 OK", &[], &invalid_quote),
        response(
            "429 Too Many Requests",
            &[("Retry-After", API_KEY)],
            "rate limit",
        ),
    ])
    .await;
    let client = make_client(&server.base_url, retry_policy(0));
    assert!(!format!("{client:?}").contains(API_KEY));
    let params = [("echo", API_KEY)];
    let provider_error = client.get::<Quote, _>("quote", &params).await.unwrap_err();
    let schema_error = client.get::<Quote, _>("quote", &params).await.unwrap_err();
    let header_error = client.get::<Quote, _>("quote", &params).await.unwrap_err();
    let escaped = serde_json::to_string(API_KEY).unwrap();
    let encoded: String = url::form_urlencoded::byte_serialize(API_KEY.as_bytes()).collect();
    for error in [provider_error, schema_error, header_error] {
        for diagnostic in [error.to_string(), format!("{error:?}")] {
            assert!(
                !diagnostic.contains(API_KEY),
                "raw API key leaked: {diagnostic}"
            );
            assert!(
                !diagnostic.contains(&escaped[1..escaped.len() - 1]),
                "JSON-escaped API key leaked: {diagnostic}"
            );
            assert!(
                !diagnostic.contains(&encoded),
                "URL-encoded API key leaked: {diagnostic}"
            );
            assert!(
                diagnostic.contains("[REDACTED]"),
                "echoed credentials must be redacted: {diagnostic}"
            );
        }
    }
    assert_eq!(finish(server).await.len(), 3);
}

#[tokio::test]
async fn unsafe_paths_credentials_and_redirects_never_contact_an_unintended_destination() {
    let trap = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let trap_address = trap.local_addr().unwrap();
    let trap_url = Url::parse(&format!("http://{trap_address}/stable/")).unwrap();
    let client = make_client(&trap_url, retry_policy(0));
    for path in [
        "https://example.com/steal",
        "//127.0.0.1/steal",
        "/quote",
        "../quote",
        "quote?apikey=secret",
        "quote#fragment",
        "quote\\escape",
        "quote/%2e%2e",
    ] {
        assert!(matches!(
            client.get_bytes(path, &[("symbol", "AAPL")]).await,
            Err(Error::Configuration { .. })
        ));
    }
    for name in ["apikey", "API_KEY", "authorization", "access-token"] {
        assert!(matches!(
            client.get_bytes("quote", &[(name, API_KEY)]).await,
            Err(Error::Configuration { .. })
        ));
    }
    let location = format!("http://{trap_address}/steal");
    let server = start_server(vec![response(
        "302 Found",
        &[("Location", &location)],
        "redirect refused",
    )])
    .await;
    let client = make_client(&server.base_url, retry_policy(0));
    let error = client
        .get_bytes("quote", &[("symbol", "AAPL")])
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Error::Api {
            status: 302,
            kind: ApiErrorKind::HttpStatus,
            ..
        }
    ));
    assert_eq!(finish(server).await.len(), 1);
    assert!(
        timeout(Duration::from_millis(100), trap.accept())
            .await
            .is_err(),
        "a rejected request or redirect reached the credential trap"
    );
}
