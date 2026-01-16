use thiserror::Error;

/// Erros possíveis durante o parsing de extratos bancários
#[derive(Error, Debug)]
pub enum StatementParseError {
    /// Falha genérica durante o parsing do conteúdo (detalhe na mensagem)
    #[error("Parse failed: {0}")]
    ParseFailed(String),

    /// Formato do arquivo não é suportado pela biblioteca
    #[error("Unsupported file format")]
    UnsupportedFormat,

    /// Erro ao ler o conteúdo do arquivo do disco
    #[error("Failed to read file content: {0}")]
    ReadContentFailed(#[from] std::io::Error),

    /// O builder foi chamado sem fornecer conteúdo nem caminho de arquivo
    #[error("Content or filepath is required")]
    MissingContentAndFilepath,

    // ── Erros específicos de formatos ───────────────────────────────────────────

    /// Data no formato QFX/OFX inválida ou malformada
    #[error("Invalid QFX/OFX date format")]
    QfxDateInvalidFormat,

    /// Data no formato CSV inválida ou em formato não reconhecido
    #[error("Invalid CSV date format")]
    CsvDateInvalidFormat,

    // Exemplos de erros que você pode adicionar no futuro:
    // #[error("Invalid amount format in CSV: {0}")]
    // CsvAmountInvalid(String),
    //
    // #[error("Missing required column in CSV: {0}")]
    // CsvMissingColumn(String),
    //
    // #[error("Invalid encoding detected")]
    // InvalidEncoding,
}

/// Alias conveniente para Result com nosso tipo de erro principal
pub type StatementResult<T> = Result<T, StatementParseError>;