pub mod gen;
pub mod model;
pub mod macros;
pub mod error;

pub(crate) const TRAIT_ATTRIBUTE_HELPER_NAME: &'static str = "enumtrait";
pub(crate) const ENUM_ATTRIBUTE_HELPER_NAME: &'static str = "traitenum";
pub(crate) const ERROR_PREFIX: &'static str = "traitenum: ";

#[macro_export]
macro_rules! span{
    () => { ::proc_macro2::Span::call_site() };
}