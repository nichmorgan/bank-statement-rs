use bank_statement_rs::ParserBuilder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if a file path was provided as a command-line argument
    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        println!("Using example CSV data from examples/sample.csv\n");
        "examples/sample.csv"
    };

    let content = std::fs::read_to_string(file_path)?;

    let transactions = ParserBuilder::new()
        .content(&content)
        .filename(file_path)
        .parse()?;

    println!("Found {} transactions\n", transactions.len());

    for (i, tx) in transactions.iter().take(10).enumerate() {
        println!("Transaction {}:", i + 1);
        println!("  Date: {}", tx.date);
        println!("  Amount: {}", tx.amount);
        println!("  Payee: {}", tx.payee.as_deref().unwrap_or("N/A"));
        println!("  Type: {}", tx.transaction_type);
        /*if let Some(payee) = &tx.payee {
            println!("  Payee: {}", payee);
        }
        if let Some(fitid) = &tx.fitid {
            println!("  FITID: {}", fitid);
        }
        if let Some(memo) = &tx.memo {
            println!("  Memo: {}", memo);
        }*/
        println!();
    }

    if transactions.len() > 10 {
        println!("... and {} more transactions", transactions.len() - 10);
    }

    Ok(())
}