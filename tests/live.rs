use std::error::Error as StdError;

use fmp_rust_sdk::{
    AccessOutcome, ApiErrorKind, Client, Error, Subscription, WebSocketEntitlements,
};

#[tokio::test]
#[ignore = "calls the real FMP service; requires FMP_API_KEY and optionally FMP_PLAN"]
async fn live_stable_quote_and_access_probe() -> Result<(), Box<dyn StdError>> {
    let client = Client::from_env(WebSocketEntitlements::none())?;
    let params = [("symbol", "AAPL")];
    let quotes = client.quote("AAPL").await?;
    assert!(
        !quotes.is_empty(),
        "the live AAPL quote request returned no records"
    );
    assert!(
        quotes
            .iter()
            .all(|quote| quote.symbol == "AAPL" && quote.price > 0.0)
    );
    let report = client.probe_access("quote", &params).await?;
    assert_eq!(report.configured_plan, client.subscription().plan());
    assert_eq!(report.outcome, AccessOutcome::Allowed);
    Ok(())
}

#[cfg(feature = "websocket")]
#[tokio::test]
#[ignore = "calls the real FMP standard WebSocket with a deliberately invalid API key"]
async fn live_invalid_standard_websocket_key_is_rejected() -> Result<(), Box<dyn StdError>> {
    use fmp_rust_sdk::websocket::{
        MarketStream, StandardWebSocketClient, StreamConfig, StreamEntitlements, WebSocketError,
    };
    use std::time::Duration;

    let stream: MarketStream = "fmp-us-equities-stream".parse()?;
    let error = StandardWebSocketClient::connect(
        "invalid-sdk-integration-test-key",
        &Subscription::new(None, WebSocketEntitlements::none()),
        StreamEntitlements::new(vec![stream])?,
        StreamConfig::new(
            Duration::from_secs(10),
            Duration::from_secs(5),
            1,
            Duration::from_millis(100),
        )?,
    )
    .await
    .expect_err("FMP must reject the deliberately invalid standard WebSocket key");
    assert!(matches!(
        error,
        WebSocketError::Server {
            status: Some(401),
            ..
        }
    ));
    Ok(())
}

#[tokio::test]
#[ignore = "calls the real FMP service with a deliberately invalid API key"]
async fn live_invalid_key_is_an_authentication_error() -> Result<(), Box<dyn StdError>> {
    let client = Client::new(
        "invalid-sdk-integration-test-key",
        Subscription::new(None, WebSocketEntitlements::none()),
    )?;
    let error = client
        .probe_access("quote", &[("symbol", "AAPL")])
        .await
        .expect_err("FMP must reject the deliberately invalid key");
    assert!(matches!(
        error,
        Error::Api {
            kind: ApiErrorKind::Authentication,
            ..
        }
    ));
    Ok(())
}

#[cfg(feature = "websocket")]
#[tokio::test]
#[ignore = "calls the real FMP stocks WebSocket with a deliberately invalid API key"]
async fn live_invalid_websocket_key_is_rejected_before_login_confirmation()
-> Result<(), Box<dyn StdError>> {
    use std::time::Duration;

    use fmp_rust_sdk::websocket::{
        ControlEvent, WebSocketClient, WebSocketCluster, WebSocketConfig, WebSocketError,
    };

    let subscription = Subscription::new(
        None,
        WebSocketEntitlements {
            stocks: true,
            crypto: false,
            forex: false,
        },
    );
    let config = WebSocketConfig::new(
        Duration::from_secs(10),
        Duration::from_secs(5),
        1,
        Duration::from_millis(100),
        |_: &ControlEvent| Ok(false),
    )?;
    let error = WebSocketClient::connect(
        "invalid-sdk-integration-test-key",
        &subscription,
        WebSocketCluster::Stocks,
        config,
    )
    .await
    .expect_err("FMP must reject the deliberately invalid WebSocket key");
    assert!(matches!(
        error,
        WebSocketError::Server {
            status: Some(401),
            ..
        }
    ));
    Ok(())
}
