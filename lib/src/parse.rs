use quote::ToTokens;
use syn::{self, parse};

use crate::model;

impl parse::Parse for model::Identifier {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut full_path: syn::Path = input.parse().map_err(|_| syn::Error::new(input.span(),
            "Unable to parse trait #enumtrait(<absolute trait path>)"))?;
        let name = full_path.segments.pop()
            .ok_or(syn::Error::new(input.span(), "enumtrait: Unable to parse trait name"))?
            .value().ident.to_string();
        let path = full_path.segments.pairs()
            .map(|pair| pair.value().ident.to_string())
            .collect();

        Ok(Self::new(path, name))
    }
}

impl TryFrom<&syn::Path> for model::ReturnType {
    type Error = String;

    fn try_from(path: &syn::Path) -> Result<Self, Self::Error> {
        let ident = match path.get_ident() {
            Some(v) => v.to_string(),
            None => return Err(format!("Unsupported return type: {}", path.to_token_stream().to_string()))
        };

        match ident.as_str() {
            "usize" => Ok(model::ReturnType::UnsignedSize),
            _ => Err(format!("Unsupported return type: {}", path.to_token_stream().to_string()))
        }
    }
}