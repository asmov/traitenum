use std::sync::OnceLock;
use syn::{self, spanned::Spanned};
use quote;

use traitenum_lib::model as model;
use traitenum_test_exporter_traits as traits;

pub fn parse_derive_exporter_traitenum(item: proc_macro2::TokenStream) -> Result<proc_macro2::TokenStream, syn::Error> {
    static MODEL: OnceLock<model::EnumTrait> = OnceLock::new();
    let model = MODEL.get_or_init(|| model::EnumTrait::from(traits::MODEL_SIMPLETRAIT as &'static [u8]));

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