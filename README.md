# bank-statement-rs

bank-statement-rs is a Rust library for parsing bank and credit card transaction history from multiple common financial export formats.

## Features

- **Multiple Format Support**: QFX/OFX with extensible architecture for CSV and other formats
- **Auto-Detection**: Automatically detect file format from content and filename
- **Builder Pattern**: Fluent API for configuring and parsing
- **Type-Safe**: Strongly-typed transactions with chrono and rust_decimal
- **Serde Support**: Serialize/deserialize transactions and format configurations

## Supported Formats

- ✅ **QFX/OFX** (both XML 2.x and SGML 1.x formats)
- 🚧 **CSV** (planned)
- 🚧 **OFX** (planned - separate from QFX)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bank-statement-rs = "0.1.0"
```

## Usage

### Builder Pattern (Recommended)

The builder pattern provides a fluent and flexible API:

```rust
use bank_statement_rs::ParserBuilder;

// Auto-detect format with filename hint
let content = std::fs::read_to_string("statement.qfx")?;
let transactions = ParserBuilder::new()
    .content(&content)
    .filename("statement.qfx")
    .parse()?;

for tx in transactions {
    println!("{} | {} | {:?}", tx.date, tx.amount, tx.payee);
}
```

### Auto-detect without filename

```rust
use bank_statement_rs::ParserBuilder;

let transactions = ParserBuilder::new()
    .content(&content)
    .parse()?;
```

### Specify format explicitly

```rust
use bank_statement_rs::{FileFormat, ParserBuilder};

let transactions = ParserBuilder::new()
    .content(&content)
    .format(FileFormat::Qfx)
    .parse()?;
```

### API Methods

The `ParserBuilder` provides the following methods:

- **`.content(&str)`** - Set the file content to parse
- **`.filename(&str)`** - Set filename for format detection (optional)
- **`.format(FileFormat)`** - Explicitly set the format to skip auto-detection (optional)
- **`.parse()`** - Parse and return `Vec<Transaction>` (the default type)
- **`.parse_into::<T>()`** - Parse and return `Vec<T>` where `T: TryFrom<ParsedTransaction>`

## Architecture

Each parser outputs its **raw format-specific structures** wrapped in a `ParsedTransaction` enum:
- QFX/OFX → `ParsedTransaction::Qfx(QfxTransaction)`
- Future parsers will add their own variants

### Default Transaction Structure

The library provides a suggested `Transaction` struct with `TryFrom<ParsedTransaction>` implemented:

```rust
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub payee: Option<String>,
    pub transaction_type: String,          // e.g., "DEBIT", "CREDIT", "CHECK"
    pub fitid: Option<String>,              // Financial Institution Transaction ID
    pub status: Option<String>,
    pub memo: Option<String>,
}
```

### Custom Output Types

You can create your own transaction structure by implementing `TryFrom<ParsedTransaction>`:

```rust
use bank_statement_rs::{ParsedTransaction, ParserBuilder};

#[derive(Debug)]
struct MyTransaction {
    amount: f64,
    merchant: String,
    date: String,
}

impl TryFrom<ParsedTransaction> for MyTransaction {
    type Error = String;

    fn try_from(parsed: ParsedTransaction) -> Result<Self, Self::Error> {
        match parsed {
            ParsedTransaction::Qfx(qfx) => Ok(MyTransaction {
                amount: qfx.amount.to_string().parse().unwrap_or(0.0),
                merchant: qfx.name.unwrap_or_default(),
                date: format!("{:?}", qfx.dt_posted),
            }),
            // Handle other formats as needed
        }
    }
}

// Use parse_into to get your custom type
let my_transactions: Vec<MyTransaction> = ParserBuilder::new()
    .content(&content)
    .parse_into()?;
```

## Extending with Custom Parsers

Implement the `Parser` trait for your custom format. Each parser should output its own raw format structure:

```rust
use bank_statement_rs::Parser;
use serde::{Deserialize, Serialize};

// Define your raw format structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTransaction {
    pub date: String,
    pub amount: String,
    pub description: String,
}

pub struct CustomParser;

impl Parser for CustomParser {
    type Output = CustomTransaction;

    fn is_supported(filename: Option<&str>, content: &str) -> bool {
        // Check if this parser can handle the file
        filename.map(|f| f.ends_with(".custom")).unwrap_or(false)
    }

    fn parse(content: &str) -> Result<Vec<CustomTransaction>, String> {
        // Parse logic here - return your raw format structures
        Ok(vec![])
    }
}

// To integrate with the builder, follow these steps:
// 1. Add a variant to ParsedTransaction enum in src/builder.rs:
//    ParsedTransaction::Custom(CustomTransaction)
// 2. Add a variant to FileFormat enum in src/builder.rs:
//    FileFormat::Custom
// 3. Update FileFormat::parse() to handle the new format
// 4. Update auto-detection logic in ParserBuilder::parse_into()
// 5. Implement TryFrom<CustomTransaction> for Transaction (optional)
```

See [CLAUDE.md](CLAUDE.md) for detailed step-by-step instructions on adding new parsers.

## Examples

See the [examples](examples/) directory for more usage examples:

```bash
# Run with example data
cargo run --example parse_qfx

# Parse your own QFX file
cargo run --example parse_qfx path/to/your/statement.qfx
```

## License

This project is open source.
