#![allow(dead_code)]
#![allow(unused_imports)]

use std::path::PathBuf;
use std::{io, fs};
use quote::{self, ToTokens, TokenStreamExt};
use syn::{self, parse, ItemTrait};
use proc_macro2;

mod model;

impl parse::Parse for model::Identifier {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut full_path: syn::Path = input.parse().map_err(|e| syn::Error::new(input.span(),
            "Unable to parse trait #enumtrait(<absolute trait path>)"))?;
        let name = full_path.segments.pop()
            .ok_or(syn::Error::new(input.span(), "Unable to parse trait name"))?
            .value().ident.to_string();
        let path = full_path.segments.pairs()
            .map(|pair| pair.value().ident.to_string())
            .collect();

        Ok(Self::new(path, name))
    }
}

#[proc_macro_attribute]
pub fn enumtrait(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let identifier = syn::parse_macro_input!(attr as model::Identifier);
    dbg!(&identifier);
    
    let mut item = syn::parse_macro_input!(item as syn::ItemTrait);
    if identifier.name() != item.ident.to_string() {
        return proc_macro::TokenStream::from(
            syn::Error::new(item.ident.span(),
                format!("Trait name does not match #traitenum(<absolute trait path>): {}", identifier.name()))
            .to_compile_error());
    }

    //dbg!(&item.ident);
    //item.items.iter().for_each(|a| { dbg!(proc_macro::TokenStream::from(a.to_token_stream())); } );

    let mut last_trait_item: Option<ItemTrait> = None;
    for trait_item in &item.items {
        match trait_item {
            syn::TraitItem::Fn(it) => {
                dbg!("Fn:");
                for a in &it.attrs {
                    dbg!(proc_macro::TokenStream::from(a.to_token_stream()));
                }

                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            syn::TraitItem::Macro(it) => {
                dbg!("Macro");
                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            syn::TraitItem::Verbatim(it) => {
                dbg!("Verbatim");
                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            syn::TraitItem::Const(it) => {
                dbg!("Const");
                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            syn::TraitItem::Type(it) => {
                dbg!("Type");
                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            _ => todo!()
        }
    }

    proc_macro::TokenStream::from(item.to_token_stream())
} 

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
            identifer: model::Identifier::new(Vec::new(), name.to_string()),
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

