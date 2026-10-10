#![cfg(feature = "websocket")]

use std::time::Duration;

use fmp_rust_sdk::{
    plan::{Plan, Subscription, WebSocketEntitlements},
    websocket::{
        ControlEvent, Heartbeat, MarketStream, StandardWebSocketClient, StreamConfig,
        StreamEntitlements, WebSocketError, WebSocketEvent,
    },
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};
use url::Url;

const TEST_KEY: &str = "standard-local-integration-key";
const DEADLINE: Duration = Duration::from_secs(3);

fn config() -> StreamConfig {
    StreamConfig::new(DEADLINE, DEADLINE, 1, Duration::ZERO)
        .expect("valid explicit stream configuration")
}

fn subscription() -> Subscription {
    Subscription::new(Some(Plan::Ultimate), WebSocketEntitlements::none())
}

fn stream() -> MarketStream {
    "fmp-us-equities-stream-delayed"
        .parse()
        .expect("documented delayed stream")
}

fn entitlements() -> StreamEntitlements {
    StreamEntitlements::new(vec![stream()]).expect("explicit stream grant")
}

async fn local_listener() -> (TcpListener, Url) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind local websocket listener");
    let address = listener.local_addr().expect("listener address");
    let url = Url::parse(&format!("ws://{address}")).expect("local websocket URL");
    (listener, url)
}

async fn receive_json(socket: &mut WebSocketStream<TcpStream>) -> Value {
    let message = timeout(DEADLINE, socket.next())
        .await
        .expect("text frame timeout")
        .expect("text frame")
        .expect("valid text frame");
    serde_json::from_str(message.to_text().expect("text frame")).expect("valid JSON frame")
}

async fn receive_login(socket: &mut WebSocketStream<TcpStream>) {
    assert_eq!(
        receive_json(socket).await,
        json!({"event":"login", "data":{"apiKey":TEST_KEY}})
    );
}

fn authorized() -> Message {
    Message::Text(
        json!({"event":"login", "status":202, "message":"ancillary envelope metadata",
            "data":{"status":200, "message":"Authenticated"}})
        .to_string()
        .into(),
    )
}

fn quote_payload() -> Value {
    json!({
        "symbol":"AAPL", "name":"Apple Inc.", "price":211.25,
        "changesPercentage":-0.12, "change":-0.25,
        "dayLow":210.5, "dayHigh":213.0, "yearLow":164.08, "yearHigh":260.1,
        "marketCap":3170000000000u64, "priceAvg50":215.2, "priceAvg200":218.6,
        "exchange":"NASDAQ", "volume":49200000u64, "avgVolume":53400000u64,
        "open":212.0, "previousClose":211.5, "timestamp":1747473420u64,
        "eps":null, "pe":null, "earningsAnnouncement":null, "sharesOutstanding":null,
        "type":"stock", "range":"164.08-260.10", "updatedAt":"ignored provider field"
    })
}

#[tokio::test]
async fn authenticates_nested_login_and_receives_standard_stream_quotes() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(tcp).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        assert!(
            timeout(Duration::from_millis(40), socket.next())
                .await
                .is_err()
        );
        socket.send(authorized()).await.expect("authorize login");
        assert_eq!(
            receive_json(&mut socket).await,
            json!({"event":"subscribe", "data":{"stream":"fmp-us-equities-stream-delayed"}})
        );
        socket
            .send(Message::Text(
                json!({"event":"heartbeat", "timestamp":1747473420002u64})
                    .to_string()
                    .into(),
            ))
            .await
            .expect("send heartbeat");
        socket
            .send(Message::Text(quote_payload().to_string().into()))
            .await
            .expect("send standard quote");
        assert_eq!(
            receive_json(&mut socket).await,
            json!({"event":"unsubscribe", "data":{"stream":"fmp-us-equities-stream-delayed"}})
        );
        let close = timeout(DEADLINE, socket.next())
            .await
            .expect("close timeout")
            .expect("close frame")
            .expect("valid close frame");
        assert!(matches!(close, Message::Close(_)));
        socket.flush().await.expect("reply to close");
    });
    let mut client = StandardWebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        entitlements(),
        endpoint,
        config(),
    )
    .await
    .expect("authenticate standard connector");
    assert!(!format!("{client:?}").contains(TEST_KEY));
    assert!(matches!(
        client.next_event().await.expect("retained login"),
        WebSocketEvent::Control(ControlEvent {
            status: Some(200),
            ..
        })
    ));
    client.subscribe(&stream()).await.expect("subscribe stream");
    let denied: MarketStream = "iex-tops".parse().expect("documented exchange stream");
    assert!(matches!(
        client.subscribe(&denied).await,
        Err(WebSocketError::StreamEntitlement { .. })
    ));
    assert!(matches!(
        client.unsubscribe(&denied).await,
        Err(WebSocketError::StreamEntitlement { .. })
    ));
    assert!(matches!(
        client.next_event().await.expect("heartbeat"),
        WebSocketEvent::Heartbeat(Heartbeat {
            timestamp: 1747473420002
        })
    ));
    match client.next_event().await.expect("standard quote") {
        WebSocketEvent::StreamQuote(quote) => {
            assert_eq!(quote.symbol, "AAPL");
            assert_eq!(quote.price, 211.25);
            assert_eq!(quote.changes_percentage, Some(-0.12));
            assert_eq!(quote.timestamp, 1747473420);
            assert_eq!(quote.market_cap, Some(3170000000000.0));
            assert_eq!(quote.eps, None);
            assert_eq!(quote.pe, None);
            assert_eq!(quote.earnings_announcement, None);
            assert_eq!(quote.shares_outstanding, None);
        }
        event => panic!("expected standard quote, received {event:?}"),
    }
    client
        .unsubscribe(&stream())
        .await
        .expect("unsubscribe stream");
    client.close().await.expect("close stream connection");
    server.await.expect("server complete");
}

#[tokio::test]
async fn rejects_nested_authentication_errors_without_exposing_api_keys() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(tcp).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket
            .send(Message::Text(
                json!({"event":"login", "data":{
                    "status":401, "message":format!("Unauthorized: {TEST_KEY}")
                }})
                .to_string()
                .into(),
            ))
            .await
            .expect("reject login");
    });
    let error = StandardWebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        entitlements(),
        endpoint,
        config(),
    )
    .await
    .expect_err("nested 401 must fail authentication");
    assert!(matches!(
        error,
        WebSocketError::Server {
            status: Some(401),
            ..
        }
    ));
    assert!(!format!("{error:?} {error}").contains(TEST_KEY));
    assert!(error.to_string().contains("[REDACTED]"));
    server.await.expect("server complete");
}

#[tokio::test]
async fn rejects_nested_subscription_errors_and_ignores_ancillary_top_level_status() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(tcp).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket.send(authorized()).await.expect("authorize login");
        assert_eq!(
            receive_json(&mut socket).await,
            json!({"event":"subscribe", "data":{"stream":"fmp-us-equities-stream-delayed"}})
        );
        socket
            .send(Message::Text(
                json!({"event":"subscribe", "status":200, "message":"ancillary envelope metadata",
                    "data":{"status":403, "message":format!("Stream access denied: {TEST_KEY}")}})
                .to_string()
                .into(),
            ))
            .await
            .expect("reject subscription");
    });
    let mut client = StandardWebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        entitlements(),
        endpoint,
        config(),
    )
    .await
    .expect("authenticate standard connector");
    client.next_event().await.expect("retained login");
    client
        .subscribe(&stream())
        .await
        .expect("send subscription");
    let error = client
        .next_event()
        .await
        .expect_err("nested 403 must reject subscription");
    assert!(matches!(
        error,
        WebSocketError::Server {
            status: Some(403),
            ..
        }
    ));
    assert!(!format!("{error:?} {error}").contains(TEST_KEY));
    assert!(error.to_string().contains("[REDACTED]"));
    server.await.expect("server complete");
}

#[tokio::test]
async fn accepts_core_quotes_and_validates_required_fields_and_optional_metadata() {
    let mut missing_timestamp = quote_payload();
    missing_timestamp
        .as_object_mut()
        .expect("quote object")
        .remove("timestamp");
    let mut missing_price = quote_payload();
    missing_price
        .as_object_mut()
        .expect("quote object")
        .remove("price");
    let mut negative_volume = quote_payload();
    negative_volume["volume"] = json!(-1.0);
    for (payload, expected_reason) in [
        (missing_timestamp, "timestamp"),
        (missing_price, "price"),
        (negative_volume, "nonnegative"),
    ] {
        let (listener, endpoint) = local_listener().await;
        let server = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.expect("accept client");
            let mut socket = accept_async(tcp).await.expect("upgrade websocket");
            receive_login(&mut socket).await;
            socket.send(authorized()).await.expect("authorize login");
            socket
                .send(Message::Text(
                    json!({"symbol":"BTCUSD", "price":60000.5, "timestamp":1747473420u64})
                        .to_string()
                        .into(),
                ))
                .await
                .expect("send core quote without asset metadata");
            socket
                .send(Message::Text(
                    json!({"symbol":"CLUSD", "price":-37.63, "timestamp":1747473420u64,
                        "name":null, "exchange":null, "volume":null, "marketCap":null})
                    .to_string()
                    .into(),
                ))
                .await
                .expect("send valid negative-price commodity quote");
            socket
                .send(Message::Text(payload.to_string().into()))
                .await
                .expect("send invalid quote");
        });
        let mut client = StandardWebSocketClient::connect_to(
            TEST_KEY,
            &subscription(),
            entitlements(),
            endpoint,
            config(),
        )
        .await
        .expect("authenticate standard connector");
        client.next_event().await.expect("retained login");
        match client.next_event().await.expect("core quote") {
            WebSocketEvent::StreamQuote(quote) => {
                assert_eq!(quote.symbol, "BTCUSD");
                assert_eq!(quote.price, 60000.5);
                assert_eq!(quote.timestamp, 1747473420);
                assert_eq!(quote.name, None);
                assert_eq!(quote.exchange, None);
                assert_eq!(quote.volume, None);
                assert_eq!(quote.changes_percentage, None);
            }
            event => panic!("expected core quote, received {event:?}"),
        }
        match client.next_event().await.expect("commodity quote") {
            WebSocketEvent::StreamQuote(quote) => {
                assert_eq!(quote.symbol, "CLUSD");
                assert_eq!(quote.price, -37.63);
                assert_eq!(quote.timestamp, 1747473420);
                assert_eq!(quote.name, None);
                assert_eq!(quote.market_cap, None);
            }
            event => panic!("expected commodity quote, received {event:?}"),
        }
        let error = client
            .next_event()
            .await
            .expect_err("invalid quote must fail");
        assert!(matches!(error, WebSocketError::Protocol { .. }));
        assert!(error.to_string().contains(expected_reason));
        server.await.expect("server complete");
    }
}

#[tokio::test]
async fn requires_complete_nested_login_payload() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(tcp).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket
            .send(Message::Text(
                json!({"event":"login", "data":{"status":200}})
                    .to_string()
                    .into(),
            ))
            .await
            .expect("send malformed login");
    });
    let error = StandardWebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        entitlements(),
        endpoint,
        config(),
    )
    .await
    .expect_err("login message must be required");
    assert!(matches!(error, WebSocketError::Protocol { .. }));
    assert!(error.to_string().contains("message"));
    server.await.expect("server complete");
}

#[tokio::test]
async fn times_out_without_nested_login_confirmation() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(tcp).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    });
    let configuration = StreamConfig::new(Duration::from_millis(70), DEADLINE, 1, Duration::ZERO)
        .expect("valid timeout configuration");
    let error = StandardWebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        entitlements(),
        endpoint,
        configuration,
    )
    .await
    .expect_err("positive authentication confirmation is required");
    assert!(matches!(
        error,
        WebSocketError::Timeout {
            operation: "login confirmation",
            ..
        }
    ));
    server.await.expect("server complete");
}
