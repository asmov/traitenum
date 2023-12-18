use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum EnumTrait {
    #[error("Top-level #[enumtrait] does not accept arguments: {0}")]
    IllegalTopLevelArguments(String),
    #[error("Attribute helper `#[enumtrait]` was used where `#[traitenum]` was expected: {0}")]
    MismatchedHelperAttribute(String),
    #[error("Associated types are not supported: {0}")]
    UnsupportedAssociatedType(String),
    #[error("Unable to parse return type for method `{0}`. {1}: {2}")]
    MethodReturnTypeParsing(String, String, String),
    #[error("Invalid definition for method `{0}`. {1} :: {2}")]
    InvalidDefinition(String, String, String)
}

#[derive(Debug, thiserror::Error)]
pub enum Model {
    // 2 is usually the entire attribute token stream
    #[error("Unable to parse definition for method `{0}`. {1} :: {2}")]
    DefinitionParsing(String, String, String),
    #[error("{0}")]
    DefinitionValidator(String),
    
}