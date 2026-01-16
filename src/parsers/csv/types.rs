use crate::errors::StatementParseError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Representa uma data extraída de um arquivo CSV de extrato bancário.
///
/// Normalmente as datas vêm em formatos como:
/// - YYYY-MM-DD
/// - DD/MM/YYYY
/// - MM/DD/YYYY (menos comum no Brasil, mas possível)
///
/// Este wrapper permite centralizar a lógica de parsing e validação.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvDate(String);

impl CsvDate {
    /// Tenta converter a string de data para `NaiveDate` aceitando os formatos mais comuns
    pub fn parse(&self) -> Result<NaiveDate, StatementParseError> {
        let s = self.0.trim();

        // Tentativas em ordem de probabilidade comum no Brasil e internacional
        if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            return Ok(date);
        }
        if let Ok(date) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
            return Ok(date);
        }
        if let Ok(date) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
            return Ok(date);
        }
        // Pode adicionar outros formatos se necessário (ex: %d-%m-%Y, etc.)

        Err(StatementParseError::CsvDateInvalidFormat)
    }
}

impl From<String> for CsvDate {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CsvDate {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<CsvDate> for NaiveDate {
    type Error = StatementParseError;

    fn try_from(date: CsvDate) -> Result<Self, Self::Error> {
        date.parse()
    }
}

// -----------------------------------------------------------------------------
// Testes
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Datelike};
    use rstest::rstest;

    #[rstest]
    #[case("2025-12-26", 2025, 12, 26)]
    #[case("26/12/2025", 2025, 12, 26)]
    #[case("12/26/2025", 2025, 12, 26)]
    #[case("2025-01-01", 2025, 1, 1)]
    #[case("31/12/2025", 2025, 12, 31)]
    fn test_csv_date_valid_formats(
        #[case] input: &str,
        #[case] year: i32,
        #[case] month: u32,
        #[case] day: u32,
    ) {
        let csv_date = CsvDate::from(input);
        let result: Result<NaiveDate, _> = csv_date.try_into();

        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), year);
        assert_eq!(date.month(), month);
        assert_eq!(date.day(), day);
    }

    #[rstest]
    #[case("2025-13-01")]     // mês inválido
    #[case("32/12/2025")]     // dia inválido
    #[case("2025-02-30")]     // fevereiro inválido
    #[case("invalid-date")]
    #[case("26-12-2025")]     // formato diferente
    #[case("")]               // vazio
    #[case("   ")]            // só espaços
    fn test_csv_date_invalid_formats(#[case] input: &str) {
        let csv_date = CsvDate::from(input);
        let result: Result<NaiveDate, _> = csv_date.try_into();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            StatementParseError::CsvDateInvalidFormat
        ));
    }

    #[test]
    fn test_csv_date_from_string() {
        let date = CsvDate::from("2025-12-26".to_string());
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_csv_date_serialization() {
        let date = CsvDate::from("26/12/2025");
        let json = serde_json::to_string(&date).unwrap();
        assert!(json.contains("26/12/2025"));

        let deserialized: CsvDate = serde_json::from_str(&json).unwrap();
        let parsed: NaiveDate = deserialized.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }

    #[test]
    fn test_trimmed_input() {
        let date = CsvDate::from("  2025-12-26  ");
        let parsed: NaiveDate = date.try_into().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 12, 26).unwrap());
    }
}