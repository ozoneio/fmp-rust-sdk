use std::path::Path;

use fmp_rust_sdk::{Client, WebSocketEntitlements};

/// Reads a local .env file without changing process environment variables.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_dotenv(Path::new(".env"), WebSocketEntitlements::none())?;
    let quotes = client.quote("AAPL").await?;
    for quote in quotes {
        println!("{}: {}", quote.symbol, quote.price);
    }
    Ok(())
}
