use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum EnumTrait {
    #[error("Top-level #[enumtrait] does not accept arguments: {0}")]
    IllegalTopLevelArguments(String)
}