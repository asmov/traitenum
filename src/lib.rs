#![allow(dead_code)]
#![allow(unused_imports)]

use std::path::PathBuf;
use std::{io, fs};
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
    let etrait = syn::parse_macro_input!(input as model::EnumTrait);
    //let input = syn::parse_macro_input!(input as syn::DeriveInput);
    //let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();*/

    println!("DEBUG");
    dbg!(&etrait);
    write_trait(&etrait);
    let etrait2 = read_trait(&etrait.identifer.name);
    dbg!(&etrait2);
    //dbg!(&input.into_token_stream());

    let output = quote::quote!{};
    proc_macro::TokenStream::from(output)
}
#[proc_macro]
pub fn loadtrait(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let etrait = read_trait("column");
    dbg!(&etrait);

    let output = quote::quote!{};
    proc_macro::TokenStream::from(output)
}
 

fn write_trait(enum_trait: &model::EnumTrait) {
    let filepath = trait_filepath(&enum_trait.identifer.name);
    dbg!(&filepath);
    let mut bufwriter = io::BufWriter::new(
        fs::File::create(filepath).unwrap());

    bincode::serialize_into(&mut bufwriter, &enum_trait).unwrap(); 
}

fn read_trait(enumtrait_identifier: &str) -> model::EnumTrait {
    let filepath = trait_filepath(enumtrait_identifier);
    let bufreader = io::BufReader::new(
        fs::File::open(filepath).unwrap());
    let enum_trait = bincode::deserialize_from(bufreader).unwrap();
    enum_trait
}


fn trait_filepath(enumtrait_identifier: &str) -> PathBuf {
    out_dir()
        .join(enumtrait_identifier.to_string()
            + ".traitenum.trait.bin")
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("ENUMTRAIT_OUT_DIR").unwrap()).canonicalize().unwrap()
}
