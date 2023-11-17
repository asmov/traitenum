pub mod gen;
pub mod model;
pub mod macros;

pub(crate) const TRAIT_ATTRIBUTE_HELPER_NAME: &'static str = "enumtrait";
pub(crate) const ENUM_ATTRIBUTE_HELPER_NAME: &'static str = "traitenum";

/// Forces a return from the current block with an Err(syn::Error). The error message is built using format!() and
/// supports variable arguments.
/// 
/// Requires that ERROR_PREFIX be in scope. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
/// 
/// Use `mksynerr!()` to simply generate the Err(syn::Error) without a `return`.
/// 
/// # Examples:
/// ```
/// # #[macro_use] extern crate traitenum_lib;
/// # use syn::spanned::Spanned;
/// # use quote::ToTokens;
/// # const ERROR_PREFIX: &'static str = "error: ";
/// # fn main() { 
/// fn strip_ident(tokens: proc_macro2::TokenStream) -> Result<syn::Ident, syn::Error> {
///     let span = tokens.span();
///     let path = match syn::parse2::<syn::Path>(tokens) {
///         Ok(p) => p,
///         Err(_) => synerr!("Unable to parse path")
///     };
/// 
///     match path.get_ident() {
///         Some(ident) => Ok(ident.to_owned()),
///         None => Err(mksynerr!("Path is not an ident: {}", path.to_token_stream().to_string()))
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! synerr {
    ($message:expr) => {
        return Err(syn::Error::new(::proc_macro2::Span::call_site(), format!("{}{}", ERROR_PREFIX, $message)))
    };
    ($message:literal, $($v:expr),+) => {
        return Err(syn::Error::new(::proc_macro2::Span::call_site(), format!("{}{}", ERROR_PREFIX, format!($message
        $(
            , $v
        )+
        ))))
    };
}

/// Creates an Err(syn::Error) object. The error message is built using format!() and supports variable arguments.
/// 
/// Requires that ERROR_PREFIX be in scope. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
/// 
/// Use `synerr!()` to force a `return` from the current block with this value.
/// 
/// # Examples
///
/// ```
/// # #[macro_use] extern crate traitenum_lib;
/// # use syn::spanned::Spanned;
/// # use quote::ToTokens;
/// # const ERROR_PREFIX: &'static str = "error: ";
/// # fn main() { 
/// fn strip_ident(tokens: proc_macro2::TokenStream) -> Result<syn::Ident, syn::Error> {
///     let span = tokens.span();
///     let path = match ::syn::parse2::<syn::Path>(tokens) {
///         Ok(p) => p,
///         Err(_) => synerr!("Unable to parse path")
///     };
/// 
///     match path.get_ident() {
///         Some(ident) => Ok(ident.to_owned()),
///         None => Err(mksynerr!("Path is not an ident: {}", path.to_token_stream().to_string()))
///     }
/// }
/// # }
/// ```
#[macro_export]
macro_rules! mksynerr {
    ($message:expr) => {
        syn::Error::new(::proc_macro2::Span::call_site(), format!("{}{}", ERROR_PREFIX, $message))
    };
    ($message:literal, $($v:expr),+) => {
        syn::Error::new(::proc_macro2::Span::call_site(), format!("{}{}", ERROR_PREFIX, format!($message
        $(
            , $v
        )+
        )))
    };
}

