use crate::{builder::ParsedTransaction, errors::StatementParseError, parsers::qfx::prelude::*};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::parsers::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub payee: Option<String>,
    pub transaction_type: String,
    pub fitid: Option<String>,
    pub status: Option<String>,
    pub memo: Option<String>,
}

impl TryFrom<ParsedTransaction> for Transaction {
    type Error = StatementParseError;

    fn try_from(parsed: ParsedTransaction) -> Result<Self, Self::Error> {
        match parsed {
            ParsedTransaction::Qfx(qfx) => qfx.try_into(),
            ParsedTransaction::Csv(csv) => csv.try_into(),
        }
    }
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = StatementParseError;

    fn try_from(stmt: CsvTransaction) -> Result<Self, Self::Error> {
        Ok(Transaction {
            date: stmt.date,
            amount: stmt.amount,
            payee: stmt.description,
            transaction_type: stmt.trn_type,
            fitid: stmt.fitid,
            status: None,
            memo: stmt.memo,
        })
    }
}

impl TryFrom<QfxTransaction> for Transaction {
    type Error = StatementParseError;

    fn try_from(stmt: QfxTransaction) -> Result<Self, Self::Error> {
        Ok(Transaction {
            date: stmt.dt_posted.try_into()?,
            amount: stmt.amount,
            payee: stmt.name,
            transaction_type: stmt.trn_type,
            fitid: stmt.fitid,
            status: None,
            memo: stmt.memo,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rstest::rstest;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn create_test_qfx_transaction() -> QfxTransaction {
        QfxTransaction {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: Decimal::from_str("-50.00").unwrap(),
            fitid: Some("202512260".to_string()),
            name: Some("Test Payee".to_string()),
            memo: Some("Test memo".to_string()),
        }
    }

    #[rstest]
    #[case(
        "DEBIT",
        "20251226120000",
        "-50.00",
        Some("202512260".to_string()),
        Some("Test Payee".to_string()),
        Some("Test memo".to_string()),
        true
    )]
    #[case("CREDIT", "20251225000000", "1500.00", None, None, None, true)]
    #[case(
        "DEBIT",
        "invalid_date",
        "-50.00",
        None,
        None,
        None,
        false  // Should fail due to invalid date
    )]
    fn test_transaction_from_qfx_transaction(
        #[case] trn_type: &str,
        #[case] dt_posted: &str,
        #[case] amount: &str,
        #[case] fitid: Option<String>,
        #[case] name: Option<String>,
        #[case] memo: Option<String>,
        #[case] should_succeed: bool,
    ) {
        let qfx = QfxTransaction {
            trn_type: trn_type.to_string(),
            dt_posted: dt_posted.into(),
            amount: Decimal::from_str(amount).unwrap(),
            fitid: fitid.clone(),
            name: name.clone(),
            memo: memo.clone(),
        };

        let result: Result<Transaction, _> = qfx.try_into();

        if should_succeed {
            assert!(result.is_ok());
            let transaction = result.unwrap();
            assert_eq!(transaction.transaction_type, trn_type);
            assert_eq!(transaction.amount, Decimal::from_str(amount).unwrap());
            assert_eq!(transaction.payee, name);
            assert_eq!(transaction.fitid, fitid);
            assert_eq!(transaction.memo, memo);
            assert_eq!(transaction.status, None);
        } else {
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_transaction_from_parsed_transaction() {
        let qfx = create_test_qfx_transaction();
        let parsed = ParsedTransaction::Qfx(qfx);

        let result: Result<Transaction, _> = parsed.try_into();
        assert!(result.is_ok());

        let transaction = result.unwrap();
        assert_eq!(transaction.transaction_type, "DEBIT");
        assert_eq!(transaction.amount, Decimal::from_str("-50.00").unwrap());
    }

    #[test]
    fn test_transaction_serialization() {
        let transaction = Transaction {
            date: NaiveDate::from_ymd_opt(2025, 12, 26).unwrap(),
            amount: Decimal::from_str("-50.00").unwrap(),
            payee: Some("Test Payee".to_string()),
            transaction_type: "DEBIT".to_string(),
            fitid: Some("202512260".to_string()),
            status: None,
            memo: Some("Test memo".to_string()),
        };

        let json = serde_json::to_string(&transaction).unwrap();
        assert!(json.contains("Test Payee"));
        assert!(json.contains("DEBIT"));

        let deserialized: Transaction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.payee, transaction.payee);
        assert_eq!(deserialized.amount, transaction.amount);
    }
}
