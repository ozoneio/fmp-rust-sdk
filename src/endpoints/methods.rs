//! Stable endpoint methods grouped behind the public query options.

use super::*;

symbol_endpoint!(
    profile,
    "profile",
    Profile,
    "Company information from Stable `profile`."
);
symbol_endpoint!(
    stock_peers,
    "stock-peers",
    Peer,
    "Comparable companies from Stable `stock-peers`."
);
symbol_endpoint!(
    quote,
    "quote",
    Quote,
    "A full quote from Stable `quote` for any supported stock, ETF, index, crypto, forex, or commodity symbol."
);
symbol_endpoint!(
    quote_short,
    "quote-short",
    ShortQuote,
    "A compact quote from Stable `quote-short`."
);
symbol_endpoint!(
    aftermarket_trade,
    "aftermarket-trade",
    AftermarketTrade,
    "The latest after-hours trade from Stable `aftermarket-trade`."
);
symbol_endpoint!(
    aftermarket_quote,
    "aftermarket-quote",
    AftermarketQuote,
    "After-hours bid and ask data from Stable `aftermarket-quote`."
);
symbol_endpoint!(
    stock_price_change,
    "stock-price-change",
    PriceChange,
    "Changes across multiple time horizons from Stable `stock-price-change`."
);
symbol_endpoint!(
    market_capitalization,
    "market-capitalization",
    MarketCapitalization,
    "Current market capitalization from Stable `market-capitalization`."
);
symbol_endpoint!(
    shares_float,
    "shares-float",
    ShareFloat,
    "Available share float from Stable `shares-float`."
);
symbol_endpoint!(
    income_statement_ttm,
    "income-statement-ttm",
    IncomeStatement,
    "Trailing twelve-month income statements from Stable `income-statement-ttm`."
);
symbol_endpoint!(
    balance_sheet_statement_ttm,
    "balance-sheet-statement-ttm",
    BalanceSheetStatement,
    "Trailing twelve-month balance sheets from Stable `balance-sheet-statement-ttm`."
);
symbol_endpoint!(
    cash_flow_statement_ttm,
    "cash-flow-statement-ttm",
    CashFlowStatement,
    "Trailing twelve-month cash flow statements from Stable `cash-flow-statement-ttm`."
);
symbol_endpoint!(
    key_metrics_ttm,
    "key-metrics-ttm",
    KeyMetricsTtm,
    "Trailing twelve-month metrics from Stable `key-metrics-ttm`."
);
symbol_endpoint!(
    ratios_ttm,
    "ratios-ttm",
    FinancialRatiosTtm,
    "Trailing twelve-month ratios from Stable `ratios-ttm`."
);
symbol_endpoint!(
    earnings,
    "earnings",
    Earnings,
    "Company earnings history and scheduled reports from Stable `earnings`."
);
symbol_endpoint!(
    dividends,
    "dividends",
    Dividend,
    "Company dividend events from Stable `dividends`."
);
symbol_endpoint!(
    splits,
    "splits",
    StockSplit,
    "Company split events from Stable `splits`."
);
symbol_endpoint!(
    price_target_consensus,
    "price-target-consensus",
    PriceTargetConsensus,
    "Consensus analyst price targets from Stable `price-target-consensus`."
);
symbol_endpoint!(
    ratings_snapshot,
    "ratings-snapshot",
    RatingSnapshot,
    "Fundamental ratings from Stable `ratings-snapshot`."
);
symbol_endpoint!(
    discounted_cash_flow,
    "discounted-cash-flow",
    DiscountedCashFlow,
    "Company DCF valuation from Stable `discounted-cash-flow`."
);
symbol_endpoint!(
    levered_discounted_cash_flow,
    "levered-discounted-cash-flow",
    DiscountedCashFlow,
    "Levered company DCF valuation from Stable `levered-discounted-cash-flow`."
);
symbol_endpoint!(
    etf_holdings,
    "etf/holdings",
    EtfHolding,
    "Underlying securities and weights from Stable `etf/holdings`."
);

batch_endpoint!(
    batch_quote,
    "batch-quote",
    Quote,
    "Full quotes for multiple symbols from Stable `batch-quote`."
);
batch_endpoint!(
    batch_quote_short,
    "batch-quote-short",
    ShortQuote,
    "Compact quotes for multiple symbols from Stable `batch-quote-short`."
);
batch_endpoint!(
    batch_aftermarket_trade,
    "batch-aftermarket-trade",
    AftermarketTrade,
    "After-hours trades for multiple symbols from Stable `batch-aftermarket-trade`."
);
batch_endpoint!(
    batch_aftermarket_quote,
    "batch-aftermarket-quote",
    AftermarketQuote,
    "After-hours quotes for multiple symbols from Stable `batch-aftermarket-quote`."
);
batch_endpoint!(
    batch_market_capitalization,
    "market-capitalization-batch",
    MarketCapitalization,
    "Market capitalizations for multiple symbols from Stable `market-capitalization-batch`."
);

directory_endpoint!(
    stock_list,
    "stock-list",
    DirectorySymbol,
    "Company symbols from Stable `stock-list`."
);
directory_endpoint!(
    etf_list,
    "etf-list",
    DirectorySymbol,
    "ETF symbols from Stable `etf-list`."
);
directory_endpoint!(
    actively_trading_list,
    "actively-trading-list",
    DirectorySymbol,
    "Actively traded symbols from Stable `actively-trading-list`."
);
directory_endpoint!(
    biggest_gainers,
    "biggest-gainers",
    MarketMover,
    "Top gaining stocks from Stable `biggest-gainers`."
);
directory_endpoint!(
    biggest_losers,
    "biggest-losers",
    MarketMover,
    "Top losing stocks from Stable `biggest-losers`."
);
directory_endpoint!(
    most_actives,
    "most-actives",
    MarketMover,
    "Most active stocks from Stable `most-actives`."
);
directory_endpoint!(
    etf_quotes,
    "batch-etf-quotes",
    Quote,
    "ETF quotes from Stable `batch-etf-quotes`."
);
directory_endpoint!(
    crypto_quotes,
    "batch-crypto-quotes",
    Quote,
    "Cryptocurrency quotes from Stable `batch-crypto-quotes`."
);
directory_endpoint!(
    forex_quotes,
    "batch-forex-quotes",
    Quote,
    "Forex quotes from Stable `batch-forex-quotes`."
);
directory_endpoint!(
    commodity_quotes,
    "batch-commodity-quotes",
    Quote,
    "Commodity quotes from Stable `batch-commodity-quotes`."
);
directory_endpoint!(
    index_quotes,
    "batch-index-quotes",
    Quote,
    "Index quotes from Stable `batch-index-quotes`."
);

statement_endpoint!(
    income_statement,
    "income-statement",
    IncomeStatement,
    "Historical income statements from Stable `income-statement`."
);
statement_endpoint!(
    balance_sheet_statement,
    "balance-sheet-statement",
    BalanceSheetStatement,
    "Historical balance sheets from Stable `balance-sheet-statement`."
);
statement_endpoint!(
    cash_flow_statement,
    "cash-flow-statement",
    CashFlowStatement,
    "Historical cash flow statements from Stable `cash-flow-statement`."
);
statement_endpoint!(
    key_metrics,
    "key-metrics",
    KeyMetrics,
    "Historical key metrics from Stable `key-metrics`."
);
statement_endpoint!(
    ratios,
    "ratios",
    FinancialRatios,
    "Historical financial ratios from Stable `ratios`."
);
statement_endpoint!(
    financial_growth,
    "financial-growth",
    FinancialGrowth,
    "Growth rates from Stable `financial-growth`."
);

impl Client {
    /// Find symbols through Stable `search-symbol`.
    pub async fn search_symbol(
        &self,
        query: &str,
        options: SearchOptions<'_>,
    ) -> Result<Vec<SearchResult>> {
        required_value("query", query)?;
        if let Some(limit) = options.limit {
            positive_limit(limit)?;
        }
        self.get(
            "search-symbol",
            &SearchQuery {
                query,
                options: &options,
            },
        )
        .await
    }

    /// Find company names through Stable `search-name`.
    pub async fn search_name(
        &self,
        query: &str,
        options: SearchOptions<'_>,
    ) -> Result<Vec<SearchResult>> {
        required_value("query", query)?;
        if let Some(limit) = options.limit {
            positive_limit(limit)?;
        }
        self.get(
            "search-name",
            &SearchQuery {
                query,
                options: &options,
            },
        )
        .await
    }

    /// Get company profiles through Stable `profile-cik`.
    pub async fn profile_by_cik(&self, cik: &str) -> Result<Vec<Profile>> {
        required_value("cik", cik)?;
        self.get("profile-cik", &[("cik", cik)]).await
    }

    /// Get all quotes for an exchange through Stable `batch-exchange-quote`.
    pub async fn exchange_quotes(&self, exchange: &str) -> Result<Vec<Quote>> {
        required_value("exchange", exchange)?;
        self.get("batch-exchange-quote", &[("exchange", exchange)])
            .await
    }

    /// Daily split-adjusted OHLCV bars from Stable `historical-price-eod/full`.
    pub async fn historical_prices(
        &self,
        symbol: &str,
        dates: DateRange,
    ) -> Result<Vec<HistoricalPrice>> {
        required_value("symbol", symbol)?;
        valid_dates(dates)?;
        self.get(
            "historical-price-eod/full",
            &HistoricalQuery { symbol, dates },
        )
        .await
    }

    /// Daily closes from Stable `historical-price-eod/light`.
    pub async fn historical_prices_light(
        &self,
        symbol: &str,
        dates: DateRange,
    ) -> Result<Vec<HistoricalPriceLight>> {
        required_value("symbol", symbol)?;
        valid_dates(dates)?;
        self.get(
            "historical-price-eod/light",
            &HistoricalQuery { symbol, dates },
        )
        .await
    }

    /// Intraday OHLCV bars at the chosen documented Stable chart interval.
    pub async fn intraday_prices(
        &self,
        symbol: &str,
        interval: IntradayInterval,
        dates: DateRange,
    ) -> Result<Vec<IntradayPrice>> {
        required_value("symbol", symbol)?;
        valid_dates(dates)?;
        self.get(interval.path(), &HistoricalQuery { symbol, dates })
            .await
    }

    /// Market cap history through Stable `historical-market-capitalization`.
    pub async fn historical_market_capitalization(
        &self,
        symbol: &str,
        dates: DateRange,
        limit: u32,
    ) -> Result<Vec<MarketCapitalization>> {
        required_value("symbol", symbol)?;
        valid_dates(dates)?;
        positive_limit(limit)?;
        self.get(
            "historical-market-capitalization",
            &HistoricalMarketCapQuery {
                symbol,
                dates,
                limit,
            },
        )
        .await
    }

    /// Analyst projections through Stable `analyst-estimates`.
    pub async fn analyst_estimates(
        &self,
        symbol: &str,
        period: EstimatePeriod,
        pagination: Pagination,
    ) -> Result<Vec<AnalystEstimate>> {
        required_value("symbol", symbol)?;
        positive_limit(pagination.limit)?;
        self.get(
            "analyst-estimates",
            &AnalystQuery {
                symbol,
                period,
                pagination,
            },
        )
        .await
    }

    /// Full earnings call content through Stable `earning-call-transcript`.
    pub async fn earnings_transcript(
        &self,
        symbol: &str,
        year: u16,
        quarter: u8,
    ) -> Result<Vec<EarningsTranscript>> {
        required_value("symbol", symbol)?;
        if year == 0 || !(1..=4).contains(&quarter) {
            return Err(Error::Configuration {
                reason: format!(
                    "earnings transcript requires a positive year and quarter 1..=4, received year={year}, quarter={quarter}"
                ),
            });
        }
        self.get(
            "earning-call-transcript",
            &TranscriptQuery {
                symbol,
                year,
                quarter,
            },
        )
        .await
    }

    /// Earnings calendar through Stable `earnings-calendar`.
    pub async fn earnings_calendar(&self, dates: DateRange) -> Result<Vec<Earnings>> {
        valid_dates(dates)?;
        self.get("earnings-calendar", &dates).await
    }

    /// Dividend calendar through Stable `dividends-calendar`.
    pub async fn dividends_calendar(&self, dates: DateRange) -> Result<Vec<Dividend>> {
        valid_dates(dates)?;
        self.get("dividends-calendar", &dates).await
    }

    /// Split calendar through Stable `splits-calendar`.
    pub async fn splits_calendar(&self, dates: DateRange) -> Result<Vec<StockSplit>> {
        valid_dates(dates)?;
        self.get("splits-calendar", &dates).await
    }

    /// Latest company news through Stable `news/stock-latest`.
    pub async fn latest_stock_news(&self, options: NewsOptions) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        self.get("news/stock-latest", &options).await
    }

    /// General news through Stable `news/general-latest`.
    pub async fn general_news(&self, options: NewsOptions) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        self.get("news/general-latest", &options).await
    }

    /// Latest press releases through Stable `news/press-releases-latest`.
    pub async fn latest_press_releases(&self, options: NewsOptions) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        self.get("news/press-releases-latest", &options).await
    }

    /// Company news selected by symbol through Stable `news/stock`.
    pub async fn stock_news(
        &self,
        symbols: &[&str],
        options: NewsOptions,
    ) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        let symbols = joined_symbols(symbols)?;
        self.get(
            "news/stock",
            &NewsQuery {
                symbols: &symbols,
                options,
            },
        )
        .await
    }

    /// Press releases selected by symbol through Stable `news/press-releases`.
    pub async fn press_releases(
        &self,
        symbols: &[&str],
        options: NewsOptions,
    ) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        let symbols = joined_symbols(symbols)?;
        self.get(
            "news/press-releases",
            &NewsQuery {
                symbols: &symbols,
                options,
            },
        )
        .await
    }

    /// Cryptocurrency news through Stable `news/crypto`.
    pub async fn crypto_news(
        &self,
        symbols: &[&str],
        options: NewsOptions,
    ) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        let symbols = joined_symbols(symbols)?;
        self.get(
            "news/crypto",
            &NewsQuery {
                symbols: &symbols,
                options,
            },
        )
        .await
    }

    /// Foreign exchange news through Stable `news/forex`.
    pub async fn forex_news(
        &self,
        symbols: &[&str],
        options: NewsOptions,
    ) -> Result<Vec<NewsArticle>> {
        valid_dates(options.dates)?;
        positive_limit(options.pagination.limit)?;
        let symbols = joined_symbols(symbols)?;
        self.get(
            "news/forex",
            &NewsQuery {
                symbols: &symbols,
                options,
            },
        )
        .await
    }

    /// Historical Treasury yields through Stable `treasury-rates`.
    pub async fn treasury_rates(&self, dates: DateRange) -> Result<Vec<TreasuryRate>> {
        valid_dates(dates)?;
        self.get("treasury-rates", &dates).await
    }

    /// A named economic series through Stable `economic-indicators`.
    /// Indicator names are supplied explicitly as documented by FMP.
    pub async fn economic_indicators(
        &self,
        name: &str,
        dates: DateRange,
    ) -> Result<Vec<EconomicIndicator>> {
        #[derive(Serialize)]
        struct Query<'a> {
            name: &'a str,
            #[serde(flatten)]
            dates: DateRange,
        }
        required_value("name", name)?;
        valid_dates(dates)?;
        self.get("economic-indicators", &Query { name, dates })
            .await
    }

    /// Scheduled economic releases through Stable `economic-calendar`.
    pub async fn economic_calendar(&self, dates: DateRange) -> Result<Vec<EconomicEvent>> {
        valid_dates(dates)?;
        self.get("economic-calendar", &dates).await
    }

    /// Sector performance for a date through Stable `sector-performance-snapshot`.
    pub async fn sector_performance(
        &self,
        date: NaiveDate,
        exchange: Option<&str>,
        sector: Option<&str>,
    ) -> Result<Vec<SectorPerformance>> {
        self.get(
            "sector-performance-snapshot",
            &SectorSnapshotQuery {
                date,
                exchange,
                sector,
            },
        )
        .await
    }

    /// Industry performance for a date through Stable `industry-performance-snapshot`.
    pub async fn industry_performance(
        &self,
        date: NaiveDate,
        exchange: Option<&str>,
        industry: Option<&str>,
    ) -> Result<Vec<IndustryPerformance>> {
        self.get(
            "industry-performance-snapshot",
            &IndustrySnapshotQuery {
                date,
                exchange,
                industry,
            },
        )
        .await
    }
}
