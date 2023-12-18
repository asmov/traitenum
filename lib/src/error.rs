use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("Top-level #[enumtrait] does not accept arguments: {0}")]
    IllegalTopLevelArguments(String),
    #[error("Attribute helper `#[enumtrait]` was used where `#[traitenum]` was expected: {0}")]
    MismatchedHelperAttribute(String),
    #[error("Associated types are not supported: {0}")]
    UnsupportedAssociatedType(String),
    /// "Unable to parse return type for method `{0}`. {1}: {2}"
    #[error("Unable to parse return type for method `{0}`. {1}: {2}")]
    MethodReturnTypeParsing(String, String, String),
    /// "Unsupported parsing of: {0} :: {1}"
    #[error("Parsing is unsupported for: {0} :: {1}")]
    UnsupportedParsing(String, String),
    /// "Unimplemented parsing of: {0} :: {1}"
    #[error("Parsing is unimplemented for: {0} :: {1}")]
    UnimplementedParsing(String, String),
    /// "Unexpected tokens found when parsing. Expected: {0} :: {1}"
    #[error("Unexpected tokens found when parsing. Expected: {0} :: {1}")]
    UnexpectedParsing(String, String),
    /// "Improper usage found when parsing. {0} :: {1}"
    #[error("Improper usage found when parsing. {0} :: {1}")]
    IllegalParsing(String, String),
    #[error("Invalid definition for method `{0}`. {1} :: {2}")]
    InvalidDefinition(String, String, String),
    /// "Unable to parse path. {0} :: {1}"
    #[error("Unable to parse path. {0} :: {1}")]
    PathParsing(String, String),
    // 2 is usually the entire attribute token stream
    #[error("Unable to parse definition for method `{0}`. {1} :: {2}")]
    DefinitionParsing(String, String, String),
    #[error("{0}")]
    DefinitionValidator(String),
}