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

    /// "Unexpected tokens found when parsing. Expected: {expected}. Found: {found} :: {tokens}"
    #[error("Unexpected tokens found when parsing. Expected: {expected}. Found: {found} :: {tokens}")]
    UnexpectedParsing{expected: String, found: String, tokens: String},

    /// "Improper usage found when parsing. {0} :: {1}"
    #[error("Improper usage found when parsing. {0} :: {1}")]
    IllegalParsing(String, String),

    /// "Invalid definition for method `{0}`. {1} :: {2}"
    #[error("Invalid definition for method `{0}`. {1} :: {2}")]
    InvalidDefinition(String, String, String),

    /// "Unable to parse path. {0} :: {1}"
    #[error("Unable to parse path. {0} :: {1}")]
    PathParsing(String, String),

    /// "Duplicate entry found when parsing {subject}: {entry} :: {tokens}"
    #[error("Duplicate entry found when parsing {subject}: {entry} :: {tokens}")]
    DuplicateParsing{ subject: String, entry: String, tokens: String },

    /// "Unable to parse definition for method `{0}`. {1} :: {2}"
    #[error("Unable to parse definition for method `{0}`. {1} :: {2}")]
    DefinitionParsing(String, String, String),

    /// "Unknown `{def_type}` definition setting `{setting}` for method `{method}` :: {tokens}"
    #[error("Unknown `{def_type}` definition setting `{setting}` for method `{method}` :: {tokens}")]
    UnknownDefinitionSetting{ method: String, def_type: String, setting: String, tokens: String },

    /// "Unknown `{def_type}` definition setting `{name}`"
    #[error("Unknown `{def_type}` definition setting `{name}`")]
    UnknownDefinitionSettingName{ def_type: String, name: String },

    /// "Unknown `{def_type}` definition preset(`{name}`)"
    #[error("Unknown `{def_type}` definition preset(`{name}`)")]
    UnknownDefinitionPresetName{ def_type: String, name: String },

    /// "Unknown definition preset(`{0}`) for method `{1}` :: {2}"
    #[error("Unknown definition preset(`{0}`) for method `{1}` :: {2}")]
    UnknownDefinitionPreset(String, String, String),

    /// "Unknown definition type `{0}` for method `{1}`` :: {2}"
    #[error("Unknown definition type `{0}` for method `{1}` :: {2}")]
    UnknownDefinitionType(String, String, String),

    /// "Invalid value for `{def_type}` definition setting `{name}(): {value}`"
    #[error("Invalid value for `{def_type}` definition setting `{setting}(): {value}`")]
    InvalidDefinitionSettingValue{ def_type: String, setting: String, value: String },

    /// "Invalid `{def_type}` definition setting `{setting}()` value for method `{method}`: {value}` :: {tokens}"
    #[error("Invalid `{def_type}` definition setting `{setting}()` value for method `{method}`: {value}` :: {tokens}")]
    InvalidDefinitionSetting{ method: String, def_type: String, setting: String, value: String, tokens: String },

    /// Generic parsing error meant to wrap syn::Error for definition parsing.
    /// 
    /// "Unable to parse {setting} of {def_type} definition for method `{method_name}`. {cause} :: {tokens}" 
    #[error("Unable to parse {setting} of {def_type} definition for method `{method_name}`. {cause} :: {tokens}")]
    DefinitionSynParsing { setting: String, def_type: String, method_name: String, cause: String, tokens: String },

    /// "{0}"
    #[error("{0}")]
    DefinitionValidator(String),
}