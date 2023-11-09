use syn::{self, spanned::Spanned};
use quote;

use traitenum_lib::model as model;

pub fn parse_derive_traitenum(
        item: proc_macro2::TokenStream,
        model_bytes: &'static [u8]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let model = model::EnumTrait::from(model_bytes);

    let item: syn::DeriveInput = syn::parse2(item)?;

    let trait_ident = syn::Ident::new(model.identifier().name(), item.span());
    let item_ident = &item.ident;

    let output = quote::quote!{
        impl #trait_ident for #item_ident {
            fn name(&self) -> &'static str {
                match self {
                    Self::Alpha => "alpha",
                    Self::Bravo => "bravo",
                    Self::Charlie => "charlie",
                }
            }
        }
    };

    Ok(output)
}