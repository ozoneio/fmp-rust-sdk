//! Receives one quote from a current FMP market stream using a project `.env`.
//!
//! Put `FMP_API_KEY` and `FMP_PLAN` in `.env`, then run
//! `cargo run --example websocket --features websocket` during feed hours.
//! The account must already have an approved `fmp-us-equities-stream` grant;
//! configure the explicitly approved stream supplied by FMP for your account.
//! A plan name alone does not grant streaming access.

use std::{error::Error, path::Path, time::Duration};

use fmp_rust_sdk::{
    Client, WebSocketEntitlements,
    websocket::{MarketStream, StreamConfig, StreamEntitlements, WebSocketEvent},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::from_dotenv(Path::new(".env"), WebSocketEntitlements::none())?;
    let market: MarketStream = "fmp-us-equities-stream".parse()?;
    let entitlements = StreamEntitlements::new(vec![market.clone()])?;
    let configuration = StreamConfig::new(
        Duration::from_secs(15),
        Duration::from_secs(60),
        3,
        Duration::from_secs(1),
    )?;
    let mut stream = client.connect_streams(entitlements, configuration).await?;
    stream.subscribe(&market).await?;
    loop {
        match stream.next_event().await? {
            WebSocketEvent::StreamQuote(quote) => {
                println!("{quote:?}");
                break;
            }
            WebSocketEvent::Market(update) => {
                println!("{update:?}");
                break;
            }
            WebSocketEvent::Heartbeat(heartbeat) => println!("{heartbeat:?}"),
            WebSocketEvent::Control(control) => println!("{control:?}"),
            WebSocketEvent::Pong(_) => {}
            WebSocketEvent::Closed(close) => {
                return Err(format!("FMP closed before the first market update: {close:?}").into());
            }
        }
    }
    stream.unsubscribe(&market).await?;
    stream.close().await?;
    Ok(())
}
