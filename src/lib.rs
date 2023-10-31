#![allow(dead_code)]
#![allow(unused_imports)]

use quote::{self, ToTokens};
use syn::{self, parse};
use proc_macro2;

mod model;

impl parse::Parse for model::EnumTrait {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let return_type: syn::Type = input.parse()?;
        dbg!(return_type.to_token_stream());
        let content;
        let brace = syn::braced!(content in input);
        let fields = content.parse_terminated(Field::parse, syn::Token![,]);
        Ok(Self {
            identifer: model::Identifier{ path: Vec::new(), name: name.to_string()},
            methods: Vec::new()
        })
    }
}

struct Field {

}

impl parse::Parse for Field {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        dbg!(key.to_token_stream());
        input.parse::<syn::Token![:]>()?;
        let value: syn::Expr = input.parse()?;
        dbg!(value.to_token_stream());
        Ok(Self {
        })
    }
}

#[proc_macro]
pub fn enumtrait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as model::EnumTrait);
    //let input = syn::parse_macro_input!(input as syn::DeriveInput);
    //let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();*/

    println!("DEBUG");
    dbg!(&input);
    //dbg!(&input.into_token_stream());

    let output = quote::quote!{};
    proc_macro::TokenStream::from(output)
}

