pub trait Parser {
    type Output;

    fn parse(content: &str) -> Result<Vec<Self::Output>, String>;

    fn is_supported(filename: Option<&str>, content: &str) -> bool;
}
