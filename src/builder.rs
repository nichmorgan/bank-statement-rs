use std::fs;

use crate::{types::Transaction, errors::StatementParseError, parsers::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedTransaction {
    Qfx(QfxTransaction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileFormat {
    #[serde(rename = "qfx")]
    Qfx,
}

impl FileFormat {
    fn parse_raw(&self, content: &str) -> Result<Vec<ParsedTransaction>, StatementParseError> {
        match self {
            FileFormat::Qfx => {
                let transactions = QfxParser::parse(content)
                    .map_err(StatementParseError::ParseFailed)?;
                Ok(transactions.into_iter().map(ParsedTransaction::Qfx).collect())
            }
        }
    }

    fn parse<T>(&self, content: &str) -> Result<Vec<T>, StatementParseError>
    where
        T: TryFrom<ParsedTransaction, Error = StatementParseError>,
    {
        self.parse_raw(content)?
            .into_iter()
            .map(T::try_from)
            .collect()
    }

    fn detect(filename: Option<&str>, content: Option<&str>) -> Result<Self, StatementParseError> {
        if let Some(content) = content {
            if QfxParser::is_supported(filename, content) {
                return Ok(FileFormat::Qfx);
            }
        }

        if let Some(filename) = filename {
            if let Some(ext) = filename.split('.').last() {
                if matches!(ext, "qfx" | "ofx") {
                    return Ok(FileFormat::Qfx);
                }
            }
        }

        Err(StatementParseError::UnsupportedFormat)
    }
}

#[derive(Default)]
pub struct ParserBuilder {
    content: Option<String>,
    filepath: Option<String>,
    format: Option<FileFormat>,
}

impl ParserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn filename(mut self, filename: &str) -> Self {
        self.filepath = Some(filename.to_string());
        self
    }

    pub fn format(mut self, format: FileFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn parse(self) -> Result<Vec<Transaction>, StatementParseError> {
        self.parse_into::<Transaction>()
    }

    pub fn parse_into<T>(self) -> Result<Vec<T>, StatementParseError>
    where
        T: TryFrom<ParsedTransaction, Error = StatementParseError>,
    {
        let format = self.format
            .map(Ok)
            .unwrap_or_else(|| FileFormat::detect(
                self.filepath.as_deref(),
                self.content.as_deref(),
            ))?;

        let content = self.content
            .map(Ok)
            .unwrap_or_else(|| {
                self.filepath
                    .ok_or(StatementParseError::MissingContentAndFilepath)
                    .and_then(|path| fs::read_to_string(path).map_err(Into::into))
            })?;

        format.parse(&content)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    const SAMPLE_QFX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
    <BANKMSGSRSV1>
        <STMTTRNRS>
            <STMTRS>
                <BANKTRANLIST>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251226120000</DTPOSTED>
                        <TRNAMT>-50.00</TRNAMT>
                        <FITID>202512260</FITID>
                        <NAME>Coffee Shop</NAME>
                        <MEMO>Morning coffee</MEMO>
                    </STMTTRN>
                </BANKTRANLIST>
            </STMTRS>
        </STMTTRNRS>
    </BANKMSGSRSV1>
</OFX>"#;

    #[test]
    fn test_builder_missing_content() {
        let result: Result<Vec<Transaction>, _> = ParserBuilder::new().parse();
        assert!(matches!(result, Err(StatementParseError::UnsupportedFormat)));
    }

    #[test]
    fn test_builder_with_format() {
        let builder = ParserBuilder::new()
            .content("test")
            .format(FileFormat::Qfx);

        assert!(builder.format.is_some());
        assert_eq!(builder.format.unwrap(), FileFormat::Qfx);
    }

    #[test]
    fn test_builder_new() {
        let builder = ParserBuilder::new();
        assert!(builder.content.is_none());
        assert!(builder.filepath.is_none());
        assert!(builder.format.is_none());
    }

    #[test]
    fn test_builder_default() {
        let builder = ParserBuilder::default();
        assert!(builder.content.is_none());
        assert!(builder.filepath.is_none());
        assert!(builder.format.is_none());
    }

    #[test]
    fn test_builder_content() {
        let builder = ParserBuilder::new().content("test content");
        assert_eq!(builder.content.unwrap(), "test content");
    }

    #[test]
    fn test_builder_filename() {
        let builder = ParserBuilder::new().filename("test.qfx");
        assert_eq!(builder.filepath.unwrap(), "test.qfx");
    }

    #[test]
    fn test_builder_chaining() {
        let builder = ParserBuilder::new()
            .content("content")
            .filename("file.qfx")
            .format(FileFormat::Qfx);

        assert!(builder.content.is_some());
        assert!(builder.filepath.is_some());
        assert!(builder.format.is_some());
    }

    #[rstest]
    #[case(Some(FileFormat::Qfx), None, "Explicit format")]
    #[case(None, None, "Auto-detect by content")]
    #[case(None, Some("statement.qfx"), "Auto-detect by filename")]
    #[case(None, Some("statement.ofx"), "Auto-detect by .ofx extension")]
    fn test_parse_with_different_detection_methods(
        #[case] format: Option<FileFormat>,
        #[case] filename: Option<&str>,
        #[case] _description: &str,
    ) {
        let mut builder = ParserBuilder::new().content(SAMPLE_QFX);

        if let Some(fmt) = format {
            builder = builder.format(fmt);
        }
        if let Some(fname) = filename {
            builder = builder.filename(fname);
        }

        let result = builder.parse();
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, "DEBIT");
    }

    #[test]
    fn test_parse_raw_to_qfx_transaction() {
        let result = FileFormat::Qfx.parse_raw(SAMPLE_QFX);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);

        match &parsed[0] {
            ParsedTransaction::Qfx(txn) => {
                assert_eq!(txn.trn_type, "DEBIT");
                assert_eq!(txn.amount, Decimal::from_str("-50.00").unwrap());
            }
        }
    }

    #[test]
    fn test_parse_into_transaction() {
        let result = ParserBuilder::new()
            .content(SAMPLE_QFX)
            .format(FileFormat::Qfx)
            .parse_into::<Transaction>();

        assert!(result.is_ok());
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, "DEBIT");
    }

    #[test]
    fn test_parse_unsupported_format() {
        let result = ParserBuilder::new()
            .content("random content that's not OFX")
            .parse();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StatementParseError::UnsupportedFormat));
    }

    #[test]
    fn test_parse_no_content_no_filepath() {
        let result = ParserBuilder::new()
            .format(FileFormat::Qfx)
            .parse();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_content() {
        let result = ParserBuilder::new()
            .content("invalid QFX content")
            .format(FileFormat::Qfx)
            .parse();

        assert!(result.is_err());
    }

    #[rstest]
    #[case(None, Some(SAMPLE_QFX), true)]  // Detect by content
    #[case(Some("statement.qfx"), None, true)]  // Detect by .qfx extension
    #[case(Some("statement.ofx"), None, true)]  // Detect by .ofx extension
    #[case(Some("statement.QFX"), Some(SAMPLE_QFX), true)]  // Case insensitive with content
    #[case(Some("statement.OFX"), Some(SAMPLE_QFX), true)]  // Case insensitive with content
    #[case(Some("statement.csv"), Some("random content"), false)]  // Unsupported
    #[case(None, None, false)]  // No input
    #[case(Some("statement.txt"), Some("not ofx"), false)]  // Unsupported content
    fn test_file_format_detect(
        #[case] filename: Option<&str>,
        #[case] content: Option<&str>,
        #[case] should_succeed: bool,
    ) {
        let result = FileFormat::detect(filename, content);
        if should_succeed {
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), FileFormat::Qfx);
        } else {
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), StatementParseError::UnsupportedFormat));
        }
    }

    #[test]
    fn test_file_format_parse_raw() {
        let result = FileFormat::Qfx.parse_raw(SAMPLE_QFX);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.len(), 1);

        match &parsed[0] {
            ParsedTransaction::Qfx(txn) => {
                assert_eq!(txn.trn_type, "DEBIT");
                assert_eq!(txn.amount, Decimal::from_str("-50.00").unwrap());
            }
        }
    }

    #[test]
    fn test_file_format_parse() {
        let result = FileFormat::Qfx.parse::<Transaction>(SAMPLE_QFX);
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].transaction_type, "DEBIT");
    }

    #[test]
    fn test_parsed_transaction_qfx_variant() {
        let qfx_txn = QfxTransaction {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: Decimal::from_str("-50.00").unwrap(),
            fitid: Some("123".to_string()),
            name: Some("Test".to_string()),
            memo: Some("Memo".to_string()),
        };

        let parsed = ParsedTransaction::Qfx(qfx_txn);

        match parsed {
            ParsedTransaction::Qfx(txn) => {
                assert_eq!(txn.trn_type, "DEBIT");
                assert_eq!(txn.amount, Decimal::from_str("-50.00").unwrap());
            }
        }
    }

    #[test]
    fn test_parsed_transaction_serialization() {
        let qfx_txn = QfxTransaction {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: Decimal::from_str("-50.00").unwrap(),
            fitid: Some("123".to_string()),
            name: Some("Test".to_string()),
            memo: None,
        };

        let parsed = ParsedTransaction::Qfx(qfx_txn);
        let json = serde_json::to_string(&parsed).unwrap();
        assert!(json.contains("DEBIT"));

        let deserialized: ParsedTransaction = serde_json::from_str(&json).unwrap();
        match deserialized {
            ParsedTransaction::Qfx(txn) => {
                assert_eq!(txn.trn_type, "DEBIT");
            }
        }
    }

    #[test]
    fn test_file_format_serialization() {
        let format = FileFormat::Qfx;
        let json = serde_json::to_string(&format).unwrap();
        assert!(json.contains("qfx"));

        let deserialized: FileFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, FileFormat::Qfx);
    }

    #[test]
    fn test_file_format_debug() {
        let format = FileFormat::Qfx;
        let debug_str = format!("{:?}", format);
        assert!(debug_str.contains("Qfx"));
    }

    #[test]
    fn test_parsed_transaction_debug() {
        let qfx_txn = QfxTransaction {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: Decimal::from_str("-50.00").unwrap(),
            fitid: None,
            name: None,
            memo: None,
        };

        let parsed = ParsedTransaction::Qfx(qfx_txn);
        let debug_str = format!("{:?}", parsed);
        assert!(debug_str.contains("Qfx"));
    }

    #[test]
    fn test_parsed_transaction_clone() {
        let qfx_txn = QfxTransaction {
            trn_type: "DEBIT".to_string(),
            dt_posted: "20251226120000".into(),
            amount: Decimal::from_str("-50.00").unwrap(),
            fitid: None,
            name: None,
            memo: None,
        };

        let parsed = ParsedTransaction::Qfx(qfx_txn);
        let cloned = parsed.clone();

        match (parsed, cloned) {
            (ParsedTransaction::Qfx(a), ParsedTransaction::Qfx(b)) => {
                assert_eq!(a.trn_type, b.trn_type);
                assert_eq!(a.amount, b.amount);
            }
        }
    }

    #[test]
    fn test_builder_parse_invalid_qfx() {
        let invalid_qfx = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
    <BANKMSGSRSV1>
        <STMTTRNRS>
            <STMTRS>
                <BANKTRANLIST>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251226120000</DTPOSTED>
                        <TRNAMT>invalid</TRNAMT>
                    </STMTTRN>
                </BANKTRANLIST>
            </STMTRS>
        </STMTTRNRS>
    </BANKMSGSRSV1>
</OFX>"#;

        let result = ParserBuilder::new()
            .content(invalid_qfx)
            .format(FileFormat::Qfx)
            .parse();

        assert!(result.is_err());
    }
}
