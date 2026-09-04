use chrono::{DateTime, Utc};

use crate::shared::errors::InvestmentError;
use crate::shared::ids::AssetID;
use crate::shared::money::Money;

/// Current market quote for an asset.
///
/// Quotes represent the latest known price for a tradeable instrument.
/// They are produced by [`QuoteProvider`] implementations on the frontend
/// side and consumed by application-layer query handlers.
#[derive(Debug, Clone)]
pub struct Quote {
    /// The asset this quote refers to.
    pub asset_id: AssetID,
    /// Ticker symbol (e.g. `"PETR4"`).
    pub ticker: String,
    /// Current per-unit price in the asset's currency.
    pub price: Money,
    /// Timestamp of the quote.
    pub timestamp: DateTime<Utc>,
}

/// Port for fetching real-time or delayed market quotes.
///
/// This is a **driven port** — implementations live on the frontend side
/// (TUI, web, Android) and are injected into application handlers.
///
/// # Example (frontend side)
///
/// ```ignore
/// struct ApiQuoteProvider { client: reqwest::Client }
///
/// #[async_trait]
/// impl QuoteProvider for ApiQuoteProvider {
///     async fn get_quote(&self, ticker: &str) -> Result<Quote, InvestmentError> { ... }
/// }
/// ```
#[async_trait::async_trait]
pub trait QuoteProvider: Send + Sync {
    /// Fetches the latest quote for a single ticker.
    async fn get_quote(&self, ticker: &str) -> Result<Quote, InvestmentError>;

    /// Fetches the latest quotes for multiple tickers in a single call.
    async fn get_quotes(&self, tickers: &[&str]) -> Result<Vec<Quote>, InvestmentError>;
}
