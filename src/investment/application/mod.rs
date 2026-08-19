//! Application layer for the investment module.
//!
//! Contains use-case handlers for registering assets, recording
//! buy/sell operations, and querying profitability and portfolio summaries.

pub mod get_portfolio_summary;
pub mod get_profitability;
pub mod record_asset_purchase;
pub mod record_asset_sale;
pub mod register_asset;
