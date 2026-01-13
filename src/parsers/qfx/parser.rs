use super::dto::{OfxXml, QfxTransaction};
use crate::parsers::traits::Parser;

pub struct QfxParser;

impl Parser for QfxParser {
    type Output = QfxTransaction;

    fn is_supported(filename: Option<&str>, content: &str) -> bool {
        if let Some(name) = filename {
            let ext = name.to_lowercase();
            if ext.ends_with(".qfx") || ext.ends_with(".ofx") {
                return true;
            }
        }

        let trimmed = content.trim();
        trimmed.contains("<OFX>")
            || trimmed.contains("OFXHEADER:")
            || trimmed.contains("DATA:OFXSGML")
    }

    fn parse(content: &str) -> Result<Vec<Self::Output>, String> {
        let xml_content = if content.trim().starts_with("<?xml") {
            content.to_string()
        } else {
            convert_sgml_to_xml(content)?
        };

        let ofx_start = xml_content.find("<OFX>").ok_or("Missing <OFX> tag")?;
        let ofx_end = xml_content.find("</OFX>").ok_or("Missing </OFX> tag")?;
        let ofx_content = &xml_content[ofx_start..=ofx_end + 5];

        let ofx: OfxXml = serde_xml_rs::from_str(ofx_content)
            .map_err(|e| format!("XML parse error: {}", e))?;

        let raw_transactions = ofx.bank_msgs
            .map(|b| b.stmt_trn_rs.stmt_rs.bank_transaction_list.transactions)
            .or_else(|| ofx.cc_msgs.map(|c| c.cc_stmt_trn_rs.cc_stmt_rs.bank_transaction_list.transactions))
            .ok_or("No transaction data found")?;

        raw_transactions
            .into_iter()
            .map(QfxTransaction::from_raw)
            .collect()
    }
}

fn convert_sgml_to_xml(content: &str) -> Result<String, String> {
    const LEAF_ELEMENTS: &[&str] = &[
        "CODE", "SEVERITY", "MESSAGE", "DTSERVER", "LANGUAGE", "ORG", "FID", "TRNUID", "CURDEF",
        "BANKID", "ACCTID", "ACCTTYPE", "DTSTART", "DTEND", "TRNTYPE", "DTPOSTED", "DTUSER",
        "TRNAMT", "FITID", "NAME", "MEMO", "INTU.BID", "DTPROFUP", "DTASOF", "BALAMT",
    ];

    let mut result = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.peek() {
        if line.contains("<OFX>") {
            break;
        }
        lines.next();
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if !trimmed.starts_with('<') || trimmed.starts_with("</") {
            result.push_str(trimmed);
            result.push('\n');
            continue;
        }

        let tag_end = trimmed
            .find(|c: char| c == '>' || c.is_whitespace())
            .unwrap_or(trimmed.len());
        let tag_name = &trimmed[1..tag_end];

        if LEAF_ELEMENTS.contains(&tag_name.to_uppercase().as_str()) {
            if let Some(content_start) = trimmed.find('>') {
                let after_tag = &trimmed[content_start + 1..];
                let closing_tag = format!("</{}>", tag_name);

                if !after_tag.contains(&closing_tag) {
                    let content_end = after_tag.find("</").unwrap_or(after_tag.len());
                    let content = after_tag[..content_end].trim();
                    let trailing = &after_tag[content_end..];

                    result.push_str(&trimmed[..content_start + 1]);
                    result.push_str(content);
                    result.push_str(&closing_tag);
                    result.push_str(trailing);
                    result.push('\n');
                    continue;
                }
            }
        }

        result.push_str(trimmed);
        result.push('\n');
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    const SAMPLE_XML_QFX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
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

    const SAMPLE_CC_XML_QFX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
    <CREDITCARDMSGSRSV1>
        <CCSTMTTRNRS>
            <CCSTMTRS>
                <BANKTRANLIST>
                    <STMTTRN>
                        <TRNTYPE>CREDIT</TRNTYPE>
                        <DTPOSTED>20251225120000</DTPOSTED>
                        <TRNAMT>1500.00</TRNAMT>
                        <FITID>202512250</FITID>
                        <NAME>ACME Corp</NAME>
                    </STMTTRN>
                </BANKTRANLIST>
            </CCSTMTRS>
        </CCSTMTTRNRS>
    </CREDITCARDMSGSRSV1>
</OFX>"#;

    const SAMPLE_SGML_QFX: &str = r#"OFXHEADER:100
DATA:OFXSGML
VERSION:102

<OFX>
<BANKMSGSRSV1>
<STMTTRNRS>
<TRNUID>1
<STMTRS>
<CURDEF>USD
<BANKTRANLIST>
<DTSTART>20251201
<DTEND>20251231
<STMTTRN>
<TRNTYPE>DEBIT
<DTPOSTED>20251226120000
<TRNAMT>-50.00
<FITID>202512260
<NAME>Coffee Shop
<MEMO>Morning coffee
</STMTTRN>
</BANKTRANLIST>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>"#;

    // Test is_supported method
    #[rstest]
    #[case(Some("test.qfx"), "", true)]
    #[case(Some("test.ofx"), "", true)]
    #[case(Some("test.QFX"), "", true)]
    #[case(Some("test.OFX"), "", true)]
    #[case(Some("test.csv"), "", false)]
    #[case(None, "<OFX>", true)]
    #[case(None, "OFXHEADER:", true)]
    #[case(None, "DATA:OFXSGML", true)]
    #[case(None, "random content", false)]
    fn test_is_supported(#[case] filename: Option<&str>, #[case] content: &str, #[case] expected: bool) {
        assert_eq!(QfxParser::is_supported(filename, content), expected);
    }

    #[test]
    fn test_parse_xml_bank_statement() {
        let result = QfxParser::parse(SAMPLE_XML_QFX);
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);

        let txn = &transactions[0];
        assert_eq!(txn.trn_type, "DEBIT");
        assert_eq!(txn.amount.to_string(), "-50.00");
        assert_eq!(txn.fitid, Some("202512260".to_string()));
        assert_eq!(txn.name, Some("Coffee Shop".to_string()));
        assert_eq!(txn.memo, Some("Morning coffee".to_string()));
    }

    #[test]
    fn test_parse_xml_credit_card_statement() {
        let result = QfxParser::parse(SAMPLE_CC_XML_QFX);
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);

        let txn = &transactions[0];
        assert_eq!(txn.trn_type, "CREDIT");
        assert_eq!(txn.amount.to_string(), "1500.00");
        assert_eq!(txn.fitid, Some("202512250".to_string()));
        assert_eq!(txn.name, Some("ACME Corp".to_string()));
        assert_eq!(txn.memo, None);
    }

    #[test]
    fn test_parse_sgml_statement() {
        let result = QfxParser::parse(SAMPLE_SGML_QFX);
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 1);

        let txn = &transactions[0];
        assert_eq!(txn.trn_type, "DEBIT");
        assert_eq!(txn.amount.to_string(), "-50.00");
        assert_eq!(txn.fitid, Some("202512260".to_string()));
        assert_eq!(txn.name, Some("Coffee Shop".to_string()));
    }

    #[test]
    fn test_parse_missing_ofx_tag() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<INVALID>
</INVALID>"#;

        let result = QfxParser::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing <OFX> tag"));
    }

    #[test]
    fn test_parse_missing_closing_ofx_tag() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
<BANKMSGSRSV1>
</BANKMSGSRSV1>"#;

        let result = QfxParser::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing </OFX> tag"));
    }

    #[test]
    fn test_parse_no_transaction_data() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
</OFX>"#;

        let result = QfxParser::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No transaction data found"));
    }

    #[test]
    fn test_parse_invalid_xml() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
<BANKMSGSRSV1>
<INVALID XML
</OFX>"#;

        let result = QfxParser::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("XML parse error"));
    }

    #[test]
    fn test_parse_invalid_amount_in_transaction() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
    <BANKMSGSRSV1>
        <STMTTRNRS>
            <STMTRS>
                <BANKTRANLIST>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251226120000</DTPOSTED>
                        <TRNAMT>invalid_amount</TRNAMT>
                        <FITID>202512260</FITID>
                    </STMTTRN>
                </BANKTRANLIST>
            </STMTRS>
        </STMTTRNRS>
    </BANKMSGSRSV1>
</OFX>"#;

        let result = QfxParser::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid amount"));
    }

    #[test]
    fn test_parse_multiple_transactions() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
<OFX>
    <BANKMSGSRSV1>
        <STMTTRNRS>
            <STMTRS>
                <BANKTRANLIST>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251226120000</DTPOSTED>
                        <TRNAMT>-50.00</TRNAMT>
                        <FITID>1</FITID>
                    </STMTTRN>
                    <STMTTRN>
                        <TRNTYPE>CREDIT</TRNTYPE>
                        <DTPOSTED>20251227120000</DTPOSTED>
                        <TRNAMT>1500.00</TRNAMT>
                        <FITID>2</FITID>
                    </STMTTRN>
                    <STMTTRN>
                        <TRNTYPE>DEBIT</TRNTYPE>
                        <DTPOSTED>20251228120000</DTPOSTED>
                        <TRNAMT>-25.00</TRNAMT>
                        <FITID>3</FITID>
                    </STMTTRN>
                </BANKTRANLIST>
            </STMTRS>
        </STMTTRNRS>
    </BANKMSGSRSV1>
</OFX>"#;

        let result = QfxParser::parse(content);
        assert!(result.is_ok());

        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 3);
        assert_eq!(transactions[0].trn_type, "DEBIT");
        assert_eq!(transactions[1].trn_type, "CREDIT");
        assert_eq!(transactions[2].trn_type, "DEBIT");
    }

    #[test]
    fn test_convert_sgml_to_xml_basic() {
        let sgml = r#"OFXHEADER:100
DATA:OFXSGML
<OFX>
<TRNTYPE>DEBIT
<TRNAMT>-50.00
</OFX>"#;

        let result = convert_sgml_to_xml(sgml);
        assert!(result.is_ok());

        let xml = result.unwrap();
        assert!(xml.contains("<TRNTYPE>DEBIT</TRNTYPE>"));
        assert!(xml.contains("<TRNAMT>-50.00</TRNAMT>"));
    }

    #[test]
    fn test_convert_sgml_to_xml_strips_header() {
        let sgml = r#"OFXHEADER:100
DATA:OFXSGML
VERSION:102
<OFX>
</OFX>"#;

        let result = convert_sgml_to_xml(sgml);
        assert!(result.is_ok());

        let xml = result.unwrap();
        assert!(!xml.contains("OFXHEADER"));
        assert!(!xml.contains("DATA:OFXSGML"));
        assert!(xml.contains("<OFX>"));
    }

    #[test]
    fn test_convert_sgml_to_xml_preserves_existing_closing_tags() {
        let sgml = r#"<OFX>
<TRNTYPE>DEBIT</TRNTYPE>
</OFX>"#;

        let result = convert_sgml_to_xml(sgml);
        assert!(result.is_ok());

        let xml = result.unwrap();
        assert_eq!(xml.matches("</TRNTYPE>").count(), 1);
    }

    #[test]
    fn test_convert_sgml_to_xml_empty_content() {
        let sgml = r#"<OFX>
<NAME>
</OFX>"#;

        let result = convert_sgml_to_xml(sgml);
        assert!(result.is_ok());

        let xml = result.unwrap();
        assert!(xml.contains("<NAME></NAME>"));
    }
}
