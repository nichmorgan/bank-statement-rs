use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::types::QfxDate;

#[derive(Debug, Deserialize)]
pub(super) struct QfxBankMsgsRsV1 {
    #[serde(rename = "STMTTRNRS")]
    pub(super) stmt_trn_rs: QfxStmtTrnRs,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxCreditCardMsgsRsV1 {
    #[serde(rename = "CCSTMTTRNRS")]
    pub(super) cc_stmt_trn_rs: QfxCcStmtTrnRs,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxStmtTrnRs {
    #[serde(rename = "STMTRS")]
    pub(super) stmt_rs: QfxStmtRs,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxCcStmtTrnRs {
    #[serde(rename = "CCSTMTRS")]
    pub(super) cc_stmt_rs: QfxCcStmtRs,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxStmtRs {
    #[serde(rename = "BANKTRANLIST")]
    pub(super) bank_transaction_list: QfxBankTransactionList,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxCcStmtRs {
    #[serde(rename = "BANKTRANLIST")]
    pub(super) bank_transaction_list: QfxBankTransactionList,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxBankTransactionList {
    #[serde(rename = "STMTTRN", default)]
    pub(super) transactions: Vec<QfxTransactionRaw>,
}

#[derive(Debug, Deserialize)]
pub(super) struct OfxXml {
    #[serde(rename = "BANKMSGSRSV1")]
    pub(super) bank_msgs: Option<QfxBankMsgsRsV1>,
    #[serde(rename = "CREDITCARDMSGSRSV1")]
    pub(super) cc_msgs: Option<QfxCreditCardMsgsRsV1>,
}

#[derive(Debug, Deserialize)]
pub(super) struct QfxTransactionRaw {
    #[serde(rename = "TRNTYPE")]
    trn_type: String,
    #[serde(rename = "DTPOSTED")]
    dt_posted: QfxDate,
    #[serde(rename = "TRNAMT")]
    amount: String,
    #[serde(rename = "FITID", default)]
    fitid: Option<String>,
    #[serde(rename = "NAME", default)]
    name: Option<String>,
    #[serde(rename = "MEMO", default)]
    memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QfxTransaction {
    #[serde(rename = "TRNTYPE")]
    pub trn_type: String,
    #[serde(rename = "DTPOSTED")]
    pub dt_posted: QfxDate,
    #[serde(rename = "TRNAMT")]
    pub amount: Decimal,
    #[serde(rename = "FITID")]
    pub fitid: Option<String>,
    #[serde(rename = "NAME")]
    pub name: Option<String>,
    #[serde(rename = "MEMO")]
    pub memo: Option<String>,
}

impl QfxTransaction {
    pub(super) fn from_raw(raw: QfxTransactionRaw) -> Result<Self, String> {
        use std::str::FromStr;
        Ok(QfxTransaction {
            trn_type: raw.trn_type,
            dt_posted: raw.dt_posted,
            amount: Decimal::from_str(&raw.amount)
                .map_err(|e| format!("Invalid amount: {}", e))?,
            fitid: raw.fitid,
            name: raw.name,
            memo: raw.memo,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::str::FromStr;

    fn create_test_raw_transaction(amount: &str) -> QfxTransactionRaw {
        QfxTransactionRaw {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: amount.to_string(),
            fitid: Some("202512260".to_string()),
            name: Some("Test Payee".to_string()),
            memo: Some("Test memo".to_string()),
        }
    }

    #[test]
    fn test_from_raw_valid_positive_amount() {
        let raw = create_test_raw_transaction("1500.00");
        let result = QfxTransaction::from_raw(raw);

        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.amount, Decimal::from_str("1500.00").unwrap());
        assert_eq!(transaction.trn_type, "DEBIT");
        assert_eq!(transaction.fitid, Some("202512260".to_string()));
        assert_eq!(transaction.name, Some("Test Payee".to_string()));
        assert_eq!(transaction.memo, Some("Test memo".to_string()));
    }

    #[test]
    fn test_from_raw_valid_negative_amount() {
        let raw = create_test_raw_transaction("-50.00");
        let result = QfxTransaction::from_raw(raw);

        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.amount, Decimal::from_str("-50.00").unwrap());
    }

    #[rstest]
    #[case("100.00")]
    #[case("-100.00")]
    #[case("0.00")]
    #[case("0")]
    #[case("9999999.99")]
    #[case("0.01")]
    fn test_from_raw_various_valid_amounts(#[case] amount: &str) {
        let raw = create_test_raw_transaction(amount);
        let result = QfxTransaction::from_raw(raw);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().amount, Decimal::from_str(amount).unwrap());
    }

    #[rstest]
    #[case("invalid")]
    #[case("abc")]
    #[case("$100.00")]
    #[case("")]
    #[case("1,000.00")]
    fn test_from_raw_invalid_amounts(#[case] amount: &str) {
        let raw = create_test_raw_transaction(amount);
        let result = QfxTransaction::from_raw(raw);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid amount"));
    }

    #[test]
    fn test_from_raw_minimal_fields() {
        let raw = QfxTransactionRaw {
            trn_type: "CREDIT".to_string(),
            dt_posted: "20251225000000".into(),
            amount: "1500.00".to_string(),
            fitid: None,
            name: None,
            memo: None,
        };

        let result = QfxTransaction::from_raw(raw);
        assert!(result.is_ok());

        let transaction = result.unwrap();
        assert_eq!(transaction.trn_type, "CREDIT");
        assert_eq!(transaction.amount, Decimal::from_str("1500.00").unwrap());
        assert_eq!(transaction.fitid, None);
        assert_eq!(transaction.name, None);
        assert_eq!(transaction.memo, None);
    }

    #[test]
    fn test_qfx_transaction_serialization() {
        let transaction = QfxTransaction {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: Decimal::from_str("-50.00").unwrap(),
            fitid: Some("202512260".to_string()),
            name: Some("Test Payee".to_string()),
            memo: Some("Test memo".to_string()),
        };

        let json = serde_json::to_string(&transaction).unwrap();
        assert!(json.contains("DEBIT"));
        assert!(json.contains("Test Payee"));

        let deserialized: QfxTransaction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.trn_type, transaction.trn_type);
        assert_eq!(deserialized.amount, transaction.amount);
        assert_eq!(deserialized.name, transaction.name);
    }
}
