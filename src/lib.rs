//! Parse bank and credit card transaction history from financial export formats.
//!
//! ```rust,ignore
//! use bank_statement_rs::ParserBuilder;
//!
//! let transactions = ParserBuilder::new()
//!     .content(&file_content)
//!     .parse()?;
//! ```

mod builder;
mod types;

pub mod errors;
pub mod parsers;

pub use builder::{FileFormat, ParsedTransaction, ParserBuilder};
pub use parsers::prelude::*;
pub use types::Transaction;
