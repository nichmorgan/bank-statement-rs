use bank_statement_rs::errors::StatementParseError;
use bank_statement_rs::{ParsedTransaction, ParserBuilder};
use chrono::NaiveDate;
use std::env;

#[derive(Debug)]
struct MyTransaction {
    date: NaiveDate,
    amount: f64,
    merchant: String,
    category: String,
}

impl TryFrom<ParsedTransaction> for MyTransaction {
    type Error = StatementParseError;

    fn try_from(parsed: ParsedTransaction) -> Result<Self, Self::Error> {
        match parsed {
            ParsedTransaction::Qfx(qfx) => {
                let category = match qfx.trn_type.as_str() {
                    "DEBIT" => "Expense",
                    "CREDIT" => "Income",
                    _ => "Other",
                };

                Ok(MyTransaction {
                    date: qfx.dt_posted.try_into()?,
                    amount: qfx.amount.to_string().parse().unwrap_or(0.0),
                    merchant: qfx.name.unwrap_or_else(|| "Unknown".to_string()),
                    category: category.to_string(),
                })
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        println!("Using example QFX data from examples/sample.qfx\n");
        "examples/sample.qfx"
    };

    let content = std::fs::read_to_string(file_path)?;

    let transactions: Vec<MyTransaction> = ParserBuilder::new().content(&content).parse_into()?;

    println!("Found {} custom transactions\n", transactions.len());

    for (i, tx) in transactions.iter().enumerate() {
        println!("Transaction {}:", i + 1);
        println!("  Date: {}", tx.date);
        println!("  Amount: ${:.2}", tx.amount);
        println!("  Merchant: {}", tx.merchant);
        println!("  Category: {}", tx.category);
        println!();
    }

    Ok(())
}
