# fmp-rust-sdk

`fmp-rust-sdk` is an asynchronous Rust client for the [Financial Modeling Prep Stable API](https://site.financialmodelingprep.com/developer/docs). Use it to retrieve market prices, company information, and financial data with typed query options and response models.

## Features

- **Typed API methods:** company search, quotes, profiles, financial statements, metrics, historical prices, news, calendars, ETF holdings, and economic data.
- **Environment and `.env` configuration:** supply `FMP_API_KEY` and optional `FMP_PLAN` from your application's chosen configuration source.
- **Subscription awareness:** configured plan metadata and endpoint access probes, with FMP remaining authoritative about permissions.
- **HTTP request handling:** shared connections, configurable timeouts, and bounded retries for transient failures.
- **Useful errors:** request context, HTTP status, and response details, with API credentials redacted from SDK diagnostics.
- **Optional WebSockets:** native market streams and legacy stock, crypto, and forex feeds for eligible Premium or Ultimate accounts with FMP-approved stream grants.
- **Additional endpoint access:** request other Stable endpoints with your own typed models, or retrieve text and binary responses.

## Supported plans

The SDK supports FMP's personal pricing plans through `FMP_PLAN`:

| Plan | `FMP_PLAN` value | Advisory API allowance |
| --- | --- | --- |
| Free / Basic | `free` or `basic` | 250 requests per day |
| Starter | `starter` | 300 requests per minute |
| Premium | `premium` | 750 requests per minute |
| Ultimate | `ultimate` | 3,000 requests per minute |

These allowances follow FMP's [published pricing](https://site.financialmodelingprep.com/pricing-plans); actual limits and dataset access depend on your account. See that page for current subscription prices.

## Quick start

Add these dependencies to your project's `Cargo.toml`:

```toml
[dependencies]
fmp-rust-sdk = "0.1.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Choose either environment variables or a `.env` file for `FMP_API_KEY` and `FMP_PLAN`. The API key is required; `FMP_PLAN` defaults to `free` when unset or blank.

### Environment variables

Set your credentials in the terminal where you run your application:

```sh
export FMP_API_KEY="your_fmp_api_key"
export FMP_PLAN="free"
```

Or in PowerShell:

```powershell
$env:FMP_API_KEY = "your_fmp_api_key"
$env:FMP_PLAN = "free"
```

Put this in `src/main.rs` and run `cargo run`:

```rust
use fmp_rust_sdk::{Client, WebSocketEntitlements};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads FMP_API_KEY and FMP_PLAN from the environment.
    let client = Client::from_env(WebSocketEntitlements::none())?;

    // Get stock quotes.
    let quotes = client.quote("AAPL").await?;
    for quote in quotes {
        println!("{}: {}", quote.symbol, quote.price);
    }

    // Get company profiles.
    let profiles = client.profile("AAPL").await?;
    for profile in profiles {
        println!("{}: {}", profile.symbol, profile.company_name);
    }
    Ok(())
}
```

### Project `.env` file

Create a `.env` file in your own project's root:

```dotenv
FMP_API_KEY=your_fmp_api_key
FMP_PLAN=free
```

Add `.env` to your project's `.gitignore`, then replace the client initialization above with:

```rust
let client = Client::from_dotenv(
    std::path::Path::new(".env"),
    WebSocketEntitlements::none(),
)?;
```

Run `cargo run` from your project's root so the relative `.env` path resolves correctly. `Client::from_dotenv` reads the specified file without changing process environment variables. `Client::from_env` reads only the process environment; it does not automatically load `.env`.

### Subscription plan

`FMP_PLAN` accepts the values in [Supported plans](#supported-plans). Both configuration methods default to the free plan (`Plan::Basic`) when this setting is unset or blank. For a paid account, set `FMP_PLAN` to the plan shown in your FMP dashboard, for example `FMP_PLAN=premium`.

This configures the SDK's plan metadata; FMP determines actual endpoint access. FMP does not provide a documented endpoint to identify the exact billing plan, and the SDK can probe access to specific endpoints.

## Usage examples

Each example below belongs inside the async `main` function from the quick start, after creating `client`. Endpoint availability, symbols, and historical coverage depend on your FMP account's permissions.

### Company search

```rust
use fmp_rust_sdk::endpoints::SearchOptions;

let options = SearchOptions {
    limit: Some(10),
    exchange: None,
};
let companies = client.search_name("Apple", options).await?;
for company in companies {
    println!("{}: {}", company.symbol, company.name);
}
let symbols = client.search_symbol("AAPL", options).await?;
println!("Ticker matches: {symbols:#?}");
let profiles = client.profile_by_cik("0000320193").await?;
println!("CIK profiles: {profiles:#?}");
```

### Quotes

```rust
let quotes = client.quote("AAPL").await?;
for quote in quotes {
    println!("{}: {}", quote.symbol, quote.price);
}
let batch = client.batch_quote(&["AAPL", "MSFT", "GOOGL"]).await?;
println!("Batch quotes: {batch:#?}");
let exchange = client.exchange_quotes("NYSE").await?;
println!("Exchange quotes: {exchange:#?}");
let changes = client.stock_price_change("AAPL").await?;
println!("Price changes: {changes:#?}");
```

`quote` also accepts supported ETF, index, crypto, forex, and commodity symbols. Dedicated methods such as `crypto_quotes`, `forex_quotes`, and `etf_quotes` retrieve quotes for an asset category.

### Company information

```rust
let profiles = client.profile("AAPL").await?;
println!("Profiles: {profiles:#?}");
let peers = client.stock_peers("AAPL").await?;
println!("Peers: {peers:#?}");
let market_cap = client.market_capitalization("AAPL").await?;
println!("Market capitalization: {market_cap:#?}");
let float = client.shares_float("AAPL").await?;
println!("Share float: {float:#?}");
```

### Financial statements, metrics, and ratios

```rust
use fmp_rust_sdk::endpoints::{Period, StatementOptions};

let annual = StatementOptions {
    period: Period::Annual,
    limit: 5,
};
let quarterly = StatementOptions {
    period: Period::Quarter,
    limit: 8,
};
let income = client.income_statement("AAPL", annual).await?;
println!("Annual income statements: {income:#?}");
let balance = client.balance_sheet_statement("AAPL", quarterly).await?;
println!("Quarterly balance sheets: {balance:#?}");
let cash_flow = client.cash_flow_statement("AAPL", annual).await?;
println!("Annual cash flow: {cash_flow:#?}");
let metrics = client.key_metrics("AAPL", annual).await?;
println!("Key metrics: {metrics:#?}");
let ratios = client.ratios("AAPL", annual).await?;
println!("Financial ratios: {ratios:#?}");
let trailing = client.income_statement_ttm("AAPL").await?;
println!("Trailing twelve-month income: {trailing:#?}");
```

### Historical and intraday prices

```rust
use fmp_rust_sdk::endpoints::{DateRange, IntradayInterval};

let dates = DateRange {
    from: Some("2025-01-01".parse()?),
    to: Some("2025-01-31".parse()?),
};
let daily = client.historical_prices("AAPL", dates).await?;
for bar in daily {
    println!("{}: close={}, volume={}", bar.date, bar.close, bar.volume);
}
let intraday = client
    .intraday_prices(
        "AAPL",
        IntradayInterval::FiveMinutes,
        DateRange {
            from: Some("2025-01-31".parse()?),
            to: Some("2025-01-31".parse()?),
        },
    )
    .await?;
println!("Five-minute bars: {intraday:#?}");
```

### News and earnings calendars

```rust
use fmp_rust_sdk::endpoints::{DateRange, NewsOptions, Pagination};

let news = client
    .stock_news(
        &["AAPL", "MSFT"],
        NewsOptions {
            dates: DateRange { from: None, to: None },
            pagination: Pagination { page: 0, limit: 10 },
        },
    )
    .await?;
for article in news {
    println!("{}: {}", article.title, article.url);
}
let earnings = client
    .earnings_calendar(DateRange {
        from: Some("2025-01-01".parse()?),
        to: Some("2025-01-31".parse()?),
    })
    .await?;
println!("Earnings calendar: {earnings:#?}");
```

### ETF holdings and market performance

```rust
let holdings = client.etf_holdings("SPY").await?;
for holding in holdings {
    println!("{}: {}%", holding.asset, holding.weight_percentage);
}
let gainers = client.biggest_gainers().await?;
println!("Biggest gainers: {gainers:#?}");
let losers = client.biggest_losers().await?;
println!("Biggest losers: {losers:#?}");
let active = client.most_actives().await?;
println!("Most active stocks: {active:#?}");
let sectors = client.sector_performance("2025-01-31".parse()?, None, None).await?;
println!("Sector performance: {sectors:#?}");
```

### Analyst estimates and valuations

```rust
use fmp_rust_sdk::endpoints::{EstimatePeriod, Pagination};

let estimates = client
    .analyst_estimates(
        "AAPL",
        EstimatePeriod::Annual,
        Pagination { page: 0, limit: 5 },
    )
    .await?;
println!("Analyst estimates: {estimates:#?}");
let targets = client.price_target_consensus("AAPL").await?;
println!("Consensus price targets: {targets:#?}");
let valuation = client.discounted_cash_flow("AAPL").await?;
println!("Discounted cash flow: {valuation:#?}");
```

### Configured plan and endpoint access

An access probe makes one real API request. It reports access to that endpoint and parameter combination separately from your configured `FMP_PLAN`.

```rust
use fmp_rust_sdk::AccessOutcome;

println!("Configured plan: {:?}", client.subscription().plan());
let report = client.probe_access("quote", &[("symbol", "AAPL")]).await?;
match report.outcome {
    AccessOutcome::Allowed => println!("Access allowed: {}", report.endpoint),
    AccessOutcome::Restricted { status, body } => {
        println!("Access restricted: {} (HTTP {status}): {body}", report.endpoint);
    }
}
```

Invalid credentials, rate limits, and service failures return errors through `?`; they do not identify a lower subscription plan.

### WebSocket streaming

WebSocket support is available for eligible **Premium and Ultimate plans**. FMP's [account guide](https://site.financialmodelingprep.com/how-to/how-to-create-a-financial-modeling-prep-account) lists WebSockets for these tiers. Actual access depends on FMP approval and the streams enabled for your account; setting `FMP_PLAN` or enabling the crate feature does not grant access by itself.

Enable the optional feature in `Cargo.toml`:

```toml
fmp-rust-sdk = { version = "0.1.1", features = ["websocket"] }
```

Configure only streams already approved for your account by FMP. A plan name alone does not grant streaming access. This example waits for one quote during feed hours, then unsubscribes and closes the connection:

```rust
use std::time::Duration;
use fmp_rust_sdk::websocket::{MarketStream, StreamConfig, StreamEntitlements, WebSocketEvent};

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
            println!("{}: {}", quote.symbol, quote.price);
            break;
        }
        WebSocketEvent::Closed(close) => {
            return Err(format!("FMP closed before a quote arrived: {close:?}").into());
        }
        event => println!("{event:?}"),
    }
}
stream.unsubscribe(&market).await?;
stream.close().await?;
```

See the [crate documentation](https://docs.rs/fmp-rust-sdk) and runnable [examples](https://github.com/ozoneio/fmp-rust-sdk/tree/main/examples) for more usage and legacy WebSocket authentication configuration.

Requires Rust 1.88 or later. This is an independent SDK; it is not affiliated with FMP.
