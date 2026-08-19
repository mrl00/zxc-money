//! Importing bounded context: statement parsing, preview, and import.
//!
//! Provides the import pipeline for ingesting financial data from external
//! sources (bank statements, CSV files, etc.) into the ledger.
//!
//! # Flow
//!
//! ```text
//! 1. Frontend parses raw file → Vec<RawTransaction> (via StatementParser)
//! 2. preview()  → detect exact duplicates
//! 3. match_candidates() → find fuzzy matches
//! 4. confirm()  → create Transaction records in the ledger
//! ```
//!
//! # Modules
//!
//! - [`domain`] — `RawTransaction` value object, `TransactionsImported` event
//! - [`application`] — `PreviewHandler`, `MatchCandidatesHandler`, `ConfirmHandler`
//! - [`facade`] — `ImportingFacade` (aggregated entry point)

pub mod application;
pub mod domain;
pub mod facade;
