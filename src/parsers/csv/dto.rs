use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CsvTransactionRaw {
    #[serde(rename = "Date")]
    pub date: String,
    #[serde(rename = "Type")]
    pub trn_type: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "FITID")]
    pub fitid: Option<String>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvTransaction {
    pub date: NaiveDate,
    pub trn_type: String,
    pub description: Option<String>,
    pub amount: Decimal,
    pub fitid: Option<String>,
    pub memo: Option<String>,
}

impl TryFrom<CsvTransactionRaw> for CsvTransaction {
    type Error = String;

    fn try_from(raw: CsvTransactionRaw) -> Result<Self, Self::Error> {
        let date = NaiveDate::parse_from_str(&raw.date, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(&raw.date, "%d/%m/%Y"))
            .map_err(|e| format!("Invalid date: {}", e))?;

        let amount = raw.amount.parse::<Decimal>().map_err(|e| format!("Invalid amount: {}", e))?;

        Ok(CsvTransaction {
            date,
            trn_type: raw.trn_type,
            description: raw.description,
            amount,
            fitid: raw.fitid,
            memo: raw.memo,
        })
    }
}