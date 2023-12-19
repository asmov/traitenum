pub mod gen;
pub mod model;
pub mod macros;

pub(crate) const TRAIT_ATTRIBUTE_HELPER_NAME: &'static str = "enumtrait";
pub(crate) const ENUM_ATTRIBUTE_HELPER_NAME: &'static str = "traitenum";
pub(crate) const ERROR_PREFIX: &'static str = "[traitenum] ";

pub fn span(source: impl quote::ToTokens) -> ::proc_macro2::Span {
    syn::spanned::Spanned::span(&source.to_token_stream()) 
}

pub fn span_site() -> ::proc_macro2::Span {
    ::proc_macro2::Span::call_site()
}

/// Creates an Err(syn::Error) object. The error message is built using format!() and supports variable arguments.
/// 
/// Requires that ERROR_PREFIX be in scope. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
/// 
/// Use `synerr!()` to force a `return` from the current block with an Err() of this value.
#[macro_export]
macro_rules! mksynerr {
    ($source:expr, $message:literal) => {
        ::syn::Error::new(crate::span($source), format!("{}{}", ERROR_PREFIX, $message))
    };
    ($source:expr, $message:literal, $($v:expr),+) => {
        ::syn::Error::new(crate::span($source), format!("{}{}", ERROR_PREFIX, format!($message
            $( , $v)+
        )))
    };
}

/// Forces a return from the current block with an Err(syn::Error). The error message is built using format!() and
/// supports variable arguments.
/// 
/// Requires that ERROR_PREFIX be in scope. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
/// 
/// Use `mksynerr!()` to simply generate a syn::Error.
#[macro_export]
macro_rules! synerr {
    ($source:expr, $message:literal) => {
        return Err(mksynerr!($source, $message))
    };
    ($source:expr, $message:literal, $($v:expr),+) => {
        return Err(::syn::Error::new(crate::span($source), format!("{}{}", ERROR_PREFIX, format!($message
            $( , $v)+
        ))))
    };
}

