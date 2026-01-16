use bank_statement_rs::ParserBuilder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        println!("Using example QFX data from examples/sample.qfx\n");
        "examples/sample.qfx"
    };

    let content = std::fs::read_to_string(file_path)?;

    let transactions = ParserBuilder::new().content(&content).parse()?;

    println!("Found {} transactions\n", transactions.len());

    for (i, tx) in transactions.iter().enumerate() {
        println!("Transaction {}:", i + 1);
        println!("  Date: {}", tx.date);
        println!("  Amount: {}", tx.amount);
        println!("  Type: {}", tx.transaction_type);
        if let Some(payee) = &tx.payee {
            println!("  Payee: {}", payee);
        }
        if let Some(fitid) = &tx.fitid {
            println!("  FITID: {}", fitid);
        }
        if let Some(memo) = &tx.memo {
            println!("  Memo: {}", memo);
        }
        println!();
    }

    Ok(())
}
