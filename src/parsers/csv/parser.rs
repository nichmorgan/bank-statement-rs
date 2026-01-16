use super::dto::{CsvTransaction, CsvTransactionRaw};
use crate::parsers::traits::Parser;
use csv::ReaderBuilder;

pub struct CsvParser;

impl Parser for CsvParser {
    type Output = CsvTransaction;

    fn is_supported(filename: Option<&str>, content: &str) -> bool {
        // 1. Verificar extensão (se filename fornecido)
        let has_csv_extension = filename
            .map(|name| name.to_lowercase().ends_with(".csv"))
            .unwrap_or(false);
        
        // 2. Verificar se conteúdo parece CSV válido
        let first_line = content.lines().next().unwrap_or("");
        let looks_like_csv = first_line.contains("Date") && first_line.contains("Amount");
        
        // 3. Retornar true APENAS se:
        //    - Tem extensão .csv E conteúdo válido, OU
        //    - Não tem filename mas conteúdo é válido
        match filename {
            Some(_) => has_csv_extension && looks_like_csv, // Exige AMBOS
            None => looks_like_csv, // Só conteúdo
        }
    }

    fn parse(content: &str) -> Result<Vec<Self::Output>, String> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(content.as_bytes());

        let mut transactions = Vec::new();

        for result in reader.deserialize::<CsvTransactionRaw>() {
            let raw = result.map_err(|e| format!("CSV deserialize error: {}", e))?;
            let txn: CsvTransaction = raw.try_into()?;
            transactions.push(txn);
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const SAMPLE_CSV: &str = r#"Date,Type,Description,Amount,FITID,Memo
2025-12-26,DEBIT,Coffee Shop,-50.00,202512260,Morning coffee
2025-12-25,CREDIT,ACME Corp Payroll,1500.00,202512250,Salary deposit
"#;

    #[rstest]
    #[case(Some("test.csv"), SAMPLE_CSV, true)]  // ✅ Extensão .csv + conteúdo válido
    #[case(Some("test.CSV"), SAMPLE_CSV, true)]  // ✅ Case insensitive + conteúdo válido
    #[case(None, SAMPLE_CSV, true)]              // ✅ Sem filename, apenas conteúdo válido
    #[case(Some("test.qfx"), "", false)]         // ❌ Extensão errada
    #[case(None, "invalid content", false)]      // ❌ Conteúdo inválido
    #[case(Some("test.csv"), "", false)]         // ❌ Extensão .csv mas vazio
    #[case(Some("test.csv"), "random text", false)] // ❌ Extensão .csv mas sem headers
    fn test_is_supported(
        #[case] filename: Option<&str>,
        #[case] content: &str,
        #[case] expected: bool,
    ) {
        assert_eq!(CsvParser::is_supported(filename, content), expected);
    }

    #[test]
    fn test_parse_valid_csv() {
        let result = CsvParser::parse(SAMPLE_CSV);
        assert!(result.is_ok());
        let txns = result.unwrap();
        assert_eq!(txns.len(), 2);
        assert_eq!(txns[0].trn_type, "DEBIT");
        assert_eq!(txns[0].amount.to_string(), "-50.00");
    }

    #[test]
    fn test_parse_invalid_csv() {
        let invalid = "Date,Amount\ninvalid,invalid";
        let result = CsvParser::parse(invalid);
        assert!(result.is_err());
    }
}