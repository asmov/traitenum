use quote::{self, ToTokens};
use syn::spanned::Spanned;
use syn;
use proc_macro2;
use bincode;

use crate::model;

const ERROR_PREFIX: &'static str = "traitenum: ";
const MODEL_PREFIX: &'static str = "TRAITENUM_";

macro_rules! err {
    ($span:expr, $message:literal) => {
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

#[derive(Debug)]
struct EnumTraitMacroOutput {
    tokens: proc_macro2::TokenStream,
    model: model::EnumTrait
}

pub fn enumtrait_macro(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
        -> Result<proc_macro2::TokenStream, syn::Error> {
    let EnumTraitMacroOutput {tokens, model} = parse_enumtrait(attr, item)?;
    let model_name = syn::Ident::new(
        &format!("{}{}", MODEL_PREFIX, model.identifier().name().to_uppercase()), tokens.span());

    let bytes = &bincode::serialize(&model).unwrap();
    let bytes_len = bytes.len();
    let bytes_literal = syn::LitByteStr::new(bytes, tokens.span());

    let output = quote::quote!{
        pub const #model_name: &'static [u8; #bytes_len] = #bytes_literal;

        #tokens
    };

    Ok(output)
}

fn parse_enumtrait(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
        -> Result<EnumTraitMacroOutput, syn::Error> {
    let identifier: model::Identifier = syn::parse2(attr)?;
    
    let mut item: syn::ItemTrait = syn::parse2(item)?;
    if identifier.name() != item.ident.to_string() {
        err!(item.ident.span(),
            "Trait name does not match #enumtrait(<absolute trait path>): {}", identifier.name());
    }

    //dbg!(&item.ident);
    //item.items.iter().for_each(|a| { dbg!(proc_macro::TokenStream::from(a.to_token_stream())); } );

    //let mut methods: Vec<model::Method> = Vec::new(); 

    for trait_item in &mut item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                // ignore functions with default implementations
                if func.default.is_some() {
                    continue;
                }

                func.attrs.clear(); //TODO

                match &func.sig.output {
                    syn::ReturnType::Default => err!(trait_item.span(),
                        "Default return types are not supported"),
                    syn::ReturnType::Type(_, return_type) => match **return_type {
                        syn::Type::Path(_) => {},
                        syn::Type::Reference(_) => {},
                        _ => todo!("unsupp"),
                    },
                }

            },
            syn::TraitItem::Type(_) => {
                todo!()
            },
            syn::TraitItem::Macro(_) => {},
            syn::TraitItem::Verbatim(_) => {},
            syn::TraitItem::Const(_) => {},
            _ => {} 
        }
    }

    /*
     
    */

    Ok(EnumTraitMacroOutput {
        tokens: item.to_token_stream(),
        model: model::EnumTrait::new(identifier, Vec::new())
    })
}

#[cfg(test)]
mod tests {
    use quote;

    #[test]
    fn test_sample_model() {
        let attribute_src = quote::quote!{
            crate::tests::MyTrait
        };

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test default parsing
                fn name(&self) -> &'static str;
                // test ordinal
                #[traitenum::variant(name(Column), type(usize), default(ordinal))]
                fn column(&self) -> usize;
                // test default implementation
                fn something_default(&self) {
                    todo!();
                }
            }
        };
        
        let model = super::parse_enumtrait(attribute_src, item_src).unwrap().model;

        assert_eq!(vec!["crate", "tests"], model.identifier().path());
        assert_eq!("MyTrait", model.identifier().name());
    }

    #[test]
    fn test_sample_model_errors() {
        let _simple_attribute_src = quote::quote!{
            crate::tests::MyTrait
        };
        let simple_item_src = quote::quote!{
            pub trait MyTrait {
                fn name(&self) -> &'static str;
            }
        };

        // test error: empty identifier
        let attribute_src = quote::quote!{};
        assert!(super::parse_enumtrait(attribute_src, simple_item_src.clone()).is_err(),
            "Empty #enumtrait(<pathspec>) should throw an Error");
        
        // test error: mismatched trait name with identifier
        let attribute_src = quote::quote!{ crate::tests::TheirTrait };
        assert!(super::parse_enumtrait(attribute_src, simple_item_src.clone()).is_err(),
            "Mismatched trait name and #enumtrait(<pathspec>) identifier should throw an Error");
    }
}
