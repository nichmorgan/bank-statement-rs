use crate::errors::StatementParseError;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct QfxDate(String);

impl<'de> Deserialize<'de> for QfxDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(QfxDate)
    }
}

impl From<String> for QfxDate {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for QfxDate {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<QfxDate> for NaiveDate {
    type Error = StatementParseError;

    fn try_from(date_str: QfxDate) -> Result<Self, Self::Error> {
        let clean = date_str.0
            .split(&['[', '.'][..])
            .next()
            .ok_or(StatementParseError::QfxDateInvalidFormat)?
            .trim();

        if clean.len() < 8 {
            return Err(StatementParseError::QfxDateInvalidFormat);
        }

        let year = clean[0..4].parse().map_err(|_| StatementParseError::QfxDateInvalidFormat)?;
        let month = clean[4..6].parse().map_err(|_| StatementParseError::QfxDateInvalidFormat)?;
        let day = clean[6..8].parse().map_err(|_| StatementParseError::QfxDateInvalidFormat)?;

        NaiveDate::from_ymd_opt(year, month, day)
            .ok_or(StatementParseError::QfxDateInvalidFormat)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("20251226120000[0:GMT]", NaiveDate::from_ymd_opt(2025, 12, 26).unwrap())]
    #[case("20251224000000.000", NaiveDate::from_ymd_opt(2025, 12, 24).unwrap())]
    #[case("20251225", NaiveDate::from_ymd_opt(2025, 12, 25).unwrap())]
    #[case("20250101000000", NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())]
    #[case("20251231235959", NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())]
    #[case("20250228000000", NaiveDate::from_ymd_opt(2025, 2, 28).unwrap())]
    fn test_parse_ofx_date(#[case] date_str: &str, #[case] expected: NaiveDate) {
        let date: QfxDate = date_str.into();
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, expected);
    }

    #[rstest]
    #[case("short")]
    #[case("1234567")]
    #[case("")]
    #[case("invalid")]
    #[case("20251301")] // Invalid month
    #[case("20250229")] // Invalid day (2025 is not a leap year)
    #[case("20250132")] // Invalid day
    #[case("abcd1226")]
    fn test_parse_ofx_date_invalid(#[case] date_str: &str) {
        let date: QfxDate = date_str.into();
        let result: Result<NaiveDate, _> = date.try_into();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StatementParseError::QfxDateInvalidFormat));
    }

    #[test]
    fn test_qfx_date_from_string() {
        let date = QfxDate::from("20251226120000".to_string());
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_qfx_date_from_str() {
        let date = QfxDate::from("20251225000000");
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 25).unwrap());
    }

    #[test]
    fn test_qfx_date_serialization() {
        let date = QfxDate::from("20251226120000[0:GMT]");
        let json = serde_json::to_string(&date).unwrap();
        assert!(json.contains("20251226120000"));

        let deserialized: QfxDate = serde_json::from_str(&json).unwrap();
        let parsed: NaiveDate = deserialized.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_qfx_date_debug() {
        let date = QfxDate::from("20251226120000");
        let debug_str = format!("{:?}", date);
        assert!(debug_str.contains("20251226120000"));
    }

    #[test]
    fn test_qfx_date_clone() {
        let date = QfxDate::from("20251226120000");
        let cloned = date.clone();
        let parsed1: NaiveDate = date.try_into().unwrap();
        let parsed2: NaiveDate = cloned.try_into().unwrap();
        assert_eq!(parsed1, parsed2);
    }

    #[test]
    fn test_parse_ofx_date_with_brackets_only() {
        let date: QfxDate = "20251226[0:GMT]".into();
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_parse_ofx_date_with_dots_only() {
        let date: QfxDate = "20251226.000".into();
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_parse_ofx_date_short_format() {
        let date: QfxDate = "20251226".into();
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_parse_ofx_date_exactly_8_chars() {
        let date: QfxDate = "20250101".into();
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    }
}
