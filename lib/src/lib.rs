pub mod gen;
pub mod model;
pub mod macros;

pub(crate) const TRAIT_ATTRIBUTE_HELPER_NAME: &'static str = "enumtrait";
pub(crate) const ENUM_ATTRIBUTE_HELPER_NAME: &'static str = "traitenum";

//Define an error prefix to use this. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
#[macro_export]
macro_rules! synerr {
    ($span:expr, $message:expr) => {
        return Err(syn::Error::new($span, format!("{}{}", ERROR_PREFIX, $message)))
    };
    ($span:expr, $message:literal, $($v:expr),+) => {
        return Err(syn::Error::new($span, format!("{}{}", ERROR_PREFIX, format!($message
        $(
            , $v
        )+
        ))))
    };
}

//Define an error prefix to use this. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
#[macro_export]
macro_rules! mksynerr {
    ($span:expr, $message:expr) => {
        syn::Error::new($span, format!("{}{}", ERROR_PREFIX, $message))
    };
    ($span:expr, $message:literal, $($v:expr),+) => {
        syn::Error::new($span, format!("{}{}", ERROR_PREFIX, format!($message
        $(
            , $v
        )+
        )))
    };
}

