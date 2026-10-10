#![cfg(feature = "websocket")]

use std::time::Duration;

use fmp_rust_sdk::{
    plan::{Plan, Subscription, WebSocketEntitlements},
    websocket::{
        ControlEvent, MarketUpdate, WebSocketClient, WebSocketCluster, WebSocketConfig,
        WebSocketError, WebSocketEvent,
    },
};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_tungstenite::{
    WebSocketStream, accept_async, accept_hdr_async,
    tungstenite::{
        Message,
        handshake::server::ErrorResponse,
        http::StatusCode,
        protocol::{CloseFrame, frame::coding::CloseCode},
    },
};
use url::Url;

const TEST_KEY: &str = "local-integration-key";
const DEADLINE: Duration = Duration::from_secs(3);

// This is the local test server's explicit authentication contract. FMP's public
// docs do not specify a successful acknowledgment; no production schema is assumed.
fn confirm_local_login(control: &ControlEvent) -> Result<bool, WebSocketError> {
    Ok(control.event == "login"
        && control.status == Some(200)
        && control.message.as_deref() == Some("local server authorized"))
}

fn config(receive_timeout: Duration, attempts: u32) -> WebSocketConfig {
    WebSocketConfig::new(
        DEADLINE,
        receive_timeout,
        attempts,
        Duration::from_millis(10),
        confirm_local_login,
    )
    .expect("valid explicit configuration")
}

fn subscription() -> Subscription {
    Subscription::new(Some(Plan::Ultimate), WebSocketEntitlements::all())
}

async fn local_listener() -> (TcpListener, Url) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind local websocket listener");
    let address = listener.local_addr().expect("listener address");
    let url = Url::parse(&format!("ws://{address}")).expect("local websocket URL");
    (listener, url)
}

fn authorized() -> Message {
    Message::Text(
        json!({"event":"login", "status":200, "message":"local server authorized"})
            .to_string()
            .into(),
    )
}

async fn receive_login(socket: &mut WebSocketStream<TcpStream>) {
    let login = timeout(DEADLINE, socket.next())
        .await
        .expect("login timeout")
        .expect("login frame")
        .expect("valid login frame");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(login.to_text().expect("text login"))
            .expect("JSON login"),
        json!({"event":"login", "data":{"apiKey":TEST_KEY}})
    );
}

// Tungstenite's callback signature fixes ErrorResponse at this upstream size.
#[allow(clippy::result_large_err)]
fn reject_first_upgrade(
    _: &tokio_tungstenite::tungstenite::handshake::server::Request,
    _: tokio_tungstenite::tungstenite::handshake::server::Response,
) -> Result<tokio_tungstenite::tungstenite::handshake::server::Response, ErrorResponse> {
    let mut response = ErrorResponse::new(Some("temporarily unavailable".to_owned()));
    *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
    Err(response)
}

#[tokio::test]
async fn authenticates_before_subscription_and_receives_typed_updates() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(stream).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        // The connector must wait for confirmation before it exposes a client.
        assert!(
            timeout(Duration::from_millis(40), socket.next())
                .await
                .is_err()
        );
        socket.send(authorized()).await.expect("authorize login");
        let subscribe = timeout(DEADLINE, socket.next())
            .await
            .expect("subscribe timeout")
            .expect("subscribe frame")
            .expect("valid subscribe frame");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(subscribe.to_text().expect("text subscribe"))
                .expect("JSON subscribe"),
            json!({"event":"subscribe", "data":{"ticker":["aapl", "msft"]}})
        );
        socket
            .send(Message::Ping(b"heartbeat".to_vec().into()))
            .await
            .expect("send ping");
        socket
            .send(Message::Text(
                json!({"s":"AAPL","t":1700000000000u64,"type":"T", "lp":190.5,"ls":3.5,
                    "ap":null,"as":null,"bp":null,"bs":null,"unrelated":"ignored"})
                .to_string()
                .into(),
            ))
            .await
            .expect("send trade");
        let pong = timeout(DEADLINE, socket.next())
            .await
            .expect("pong timeout")
            .expect("pong frame")
            .expect("valid pong");
        assert_eq!(pong, Message::Pong(b"heartbeat".to_vec().into()));
        for payload in [
            json!({"s":"AAPL","t":1700000000001u64,"type":"Q", "ap":191.0,"as":4.0,
                "bp":190.0,"bs":5.0,"lp":null,"ls":null}),
            json!({"s":"AAPL","t":1700000000002u64,"type":"B","lp":190.5,"ls":3.5}),
        ] {
            socket
                .send(Message::Text(payload.to_string().into()))
                .await
                .expect("send market update");
        }
        let unsubscribe = timeout(DEADLINE, socket.next())
            .await
            .expect("unsubscribe timeout")
            .expect("unsubscribe frame")
            .expect("valid unsubscribe frame");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                unsubscribe.to_text().expect("text unsubscribe")
            )
            .expect("JSON unsubscribe"),
            json!({"event":"unsubscribe", "data":{"ticker":["aapl"]}})
        );
        let close = timeout(DEADLINE, socket.next())
            .await
            .expect("close timeout")
            .expect("close frame")
            .expect("valid close frame");
        assert!(matches!(close, Message::Close(_)));
        socket.flush().await.expect("reply to close");
    });
    let mut client = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        WebSocketCluster::Stocks,
        endpoint,
        config(DEADLINE, 1),
    )
    .await
    .expect("authenticate connector");
    assert!(!format!("{client:?}").contains(TEST_KEY));
    assert!(matches!(
        client.next_event().await.expect("retained login control"),
        WebSocketEvent::Control(ControlEvent {
            status: Some(200),
            ..
        })
    ));
    client
        .subscribe(&["AAPL", "MSFT"])
        .await
        .expect("subscribe");
    match client.next_event().await.expect("trade") {
        WebSocketEvent::Market(MarketUpdate::Trade(trade)) => {
            assert_eq!(trade.symbol, "AAPL");
            assert_eq!(trade.timestamp, 1700000000000);
            assert_eq!(trade.price, 190.5);
            assert_eq!(trade.size, 3.5);
        }
        event => panic!("expected trade, received {event:?}"),
    }
    match client.next_event().await.expect("quote") {
        WebSocketEvent::Market(MarketUpdate::Quote(quote)) => {
            assert_eq!(quote.ask_price, 191.0);
            assert_eq!(quote.bid_size, 5.0);
        }
        event => panic!("expected quote, received {event:?}"),
    }
    assert!(matches!(
        client.next_event().await.expect("trade break"),
        WebSocketEvent::Market(MarketUpdate::Break(_))
    ));
    client.unsubscribe(&["AAPL"]).await.expect("unsubscribe");
    client.close().await.expect("clean close");
    server.await.expect("server complete");
}

#[tokio::test]
async fn reports_actual_login_rejection_shape_without_leaking_credentials() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(stream).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket
            .send(Message::Text(
                json!({"event":"login", "status":401, "message":format!("Unauthorized: {TEST_KEY}")})
                    .to_string()
                    .into(),
            ))
            .await
            .expect("send actual FMP rejection shape");
    });
    let error = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        WebSocketCluster::Crypto,
        endpoint,
        config(DEADLINE, 1),
    )
    .await
    .expect_err("login must reject 401");
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
async fn fails_login_that_never_receives_positive_confirmation() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(stream).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket
            .send(Message::Text(
                json!({"event":"connected"}).to_string().into(),
            ))
            .await
            .expect("send greeting without login confirmation");
        tokio::time::sleep(Duration::from_millis(150)).await;
    });
    let configuration = WebSocketConfig::new(
        Duration::from_millis(70),
        DEADLINE,
        1,
        Duration::ZERO,
        confirm_local_login,
    )
    .expect("valid configuration");
    let error = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        WebSocketCluster::Forex,
        endpoint,
        configuration,
    )
    .await
    .expect_err("greeting must not authorize");
    assert!(matches!(
        error,
        WebSocketError::Timeout {
            operation: "login confirmation",
            ..
        }
    ));
    server.await.expect("server complete");
}

#[tokio::test]
async fn rejects_missing_required_market_fields_and_invalid_subscriptions() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(stream).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket.send(authorized()).await.expect("authorize login");
        socket
            .send(Message::Text(
                json!({"s":"AAPL", "t":1700000000000u64, "type":"T", "ls":3.0})
                    .to_string()
                    .into(),
            ))
            .await
            .expect("send incomplete market update");
    });
    let mut client = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        WebSocketCluster::Stocks,
        endpoint,
        config(DEADLINE, 1),
    )
    .await
    .expect("authenticate");
    client.next_event().await.expect("login control");
    assert!(matches!(
        client.subscribe(&[]).await,
        Err(WebSocketError::Configuration { .. })
    ));
    assert!(matches!(
        client.subscribe(&[" AAPL"]).await,
        Err(WebSocketError::Configuration { .. })
    ));
    let error = client.next_event().await.expect_err("lp is required");
    assert!(matches!(error, WebSocketError::Protocol { .. }));
    assert!(error.to_string().contains("missing field `lp`"));
    server.await.expect("server complete");
}

#[tokio::test]
async fn retries_transient_http_upgrade_failure_then_authenticates() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept first attempt");
        let rejection = accept_hdr_async(stream, reject_first_upgrade).await;
        assert!(rejection.is_err());
        let (stream, _) = timeout(DEADLINE, listener.accept())
            .await
            .expect("retry timeout")
            .expect("accept retry");
        let mut socket = accept_async(stream).await.expect("upgrade retry");
        receive_login(&mut socket).await;
        socket.send(authorized()).await.expect("authorize login");
        let close = timeout(DEADLINE, socket.next())
            .await
            .expect("close timeout")
            .expect("close frame")
            .expect("valid close frame");
        assert!(matches!(close, Message::Close(_)));
        socket.flush().await.expect("close reply");
    });
    let client = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        WebSocketCluster::Stocks,
        endpoint,
        config(DEADLINE, 2),
    )
    .await
    .expect("second attempt succeeds");
    client.close().await.expect("close retry connection");
    server.await.expect("server complete");
}

#[tokio::test]
async fn honors_remote_close_and_requires_explicit_reconnection() {
    let (listener, endpoint) = local_listener().await;
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept client");
        let mut socket = accept_async(stream).await.expect("upgrade websocket");
        receive_login(&mut socket).await;
        socket.send(authorized()).await.expect("authorize login");
        socket
            .close(Some(CloseFrame {
                code: CloseCode::Normal,
                reason: "finished".into(),
            }))
            .await
            .expect("send remote close");
        let close_reply = timeout(DEADLINE, socket.next())
            .await
            .expect("close reply timeout")
            .expect("close reply frame")
            .expect("valid close reply");
        assert!(matches!(close_reply, Message::Close(_)));
    });
    let mut client = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription(),
        WebSocketCluster::Stocks,
        endpoint,
        config(DEADLINE, 1),
    )
    .await
    .expect("authenticate");
    client.next_event().await.expect("login control");
    match client.next_event().await.expect("remote close") {
        WebSocketEvent::Closed(close) => {
            assert_eq!(close.code, Some(1000));
            assert_eq!(close.reason, "finished");
        }
        event => panic!("expected remote close, received {event:?}"),
    }
    assert!(matches!(
        client.subscribe(&["AAPL"]).await,
        Err(WebSocketError::Closed { .. })
    ));
    client.close().await.expect("already closed");
    server.await.expect("server complete");
}

#[tokio::test]
async fn blocks_ungranted_clusters_before_opening_a_connection() {
    let (listener, endpoint) = local_listener().await;
    let subscription = Subscription::new(Some(Plan::Ultimate), WebSocketEntitlements::none());
    let error = WebSocketClient::connect_to(
        TEST_KEY,
        &subscription,
        WebSocketCluster::Crypto,
        endpoint,
        config(DEADLINE, 1),
    )
    .await
    .expect_err("plan does not imply a cluster grant");
    assert!(matches!(
        error,
        WebSocketError::Entitlement {
            cluster: WebSocketCluster::Crypto
        }
    ));
    assert!(
        timeout(Duration::from_millis(30), listener.accept())
            .await
            .is_err()
    );
}
