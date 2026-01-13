use bank_statement_rs::ParserBuilder;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if a file path was provided as a command-line argument
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Parse file from command line argument
        let file_path = &args[1];
        println!("Parsing QFX file: {}\n", file_path);

        let content = std::fs::read_to_string(file_path)?;

        let transactions = ParserBuilder::new()
            .content(&content)
            .filename(file_path)
            .parse()?;

        println!("Found {} transactions\n", transactions.len());

        // Show first 10 transactions
        for (i, tx) in transactions.iter().take(10).enumerate() {
            println!("Transaction {}:", i + 1);
            println!("  Date: {}", tx.date);
            println!("  Type: {}", tx.transaction_type);
            println!("  Amount: {}", tx.amount);
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

        if transactions.len() > 10 {
            println!("... and {} more transactions", transactions.len() - 10);
        }
    } else {
        // Use example data
        println!("Using example QFX data from examples/sample.qfx\n");
        println!("Usage: cargo run --example parse_qfx [path/to/file.qfx]\n");

        let qfx_content = std::fs::read_to_string("examples/sample.qfx")?;

        println!("Example 1: Auto-detect format with filename");
        let transactions = ParserBuilder::new()
            .content(&qfx_content)
            .filename("statement.qfx")
            .parse()?;

        println!("Found {} transactions", transactions.len());
        for tx in &transactions {
            println!(
                "  {} | {} | {} | {}",
                tx.date,
                tx.amount,
                tx.payee.as_deref().unwrap_or("N/A"),
                tx.transaction_type
            );
        }

        println!("\nExample 2: Auto-detect format without filename");
        let transactions = ParserBuilder::new().content(&qfx_content).parse()?;

        println!("Found {} transactions", transactions.len());

        println!("\nExample 3: Specify format explicitly");
        let transactions = ParserBuilder::new()
            .content(&qfx_content)
            .format(bank_statement_rs::FileFormat::Qfx)
            .parse()?;

        println!("Found {} transactions", transactions.len());
    }

    Ok(())
}
