use quote::{self, ToTokens};
use syn::spanned::Spanned;
use syn;
use proc_macro2;
use bincode;

use crate::model;

const ERROR_PREFIX: &'static str = "traitenum: ";
const MODEL_PREFIX: &'static str = "TRAITENUM_";
const ATTRIBUTE_HELPER_NAME: &'static str = "traitenum";

macro_rules! err {
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

macro_rules! mkerr {
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

    let mut methods: Vec<model::Method> = Vec::new(); 

    for trait_item in &item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                // ignore functions with default implementations
                if func.default.is_some() {
                    continue;
                }

                let mut return_type: Option<model::ReturnType> = None;

                match &func.sig.output {
                    syn::ReturnType::Default => err!(trait_item.span(),
                        "Default return types () are not supported"),
                    syn::ReturnType::Type(_, ref returntype) => match **returntype {
                        syn::Type::Path(ref path_type) => {
                            return_type = match model::ReturnType::try_from(&path_type.path) {
                                Ok(v) => Some(v),
                                Err(e) => err!(trait_item.span(), e)
                            };
                        },
                        syn::Type::Reference(ref ref_type) => {
                            // only ellided and static lifetimes are supported
                            let _has_static_lifetime = match &ref_type.lifetime {
                                Some(lifetime) => {
                                    if "static" == lifetime.ident.to_string() {
                                        true
                                    } else {
                                        err!(trait_item.span(),
                                            "Only ellided and static lifetimes are supported for return types")
                                    }
                                },
                                None => false
                            };

                            // mutability isn not supported
                            if ref_type.mutability.is_some() {
                                err!(trait_item.span(), "Mutable return types are not supported")
                            }

                            match *ref_type.elem {
                                syn::Type::Path(ref path_type) => {
                                    if let Some(ident) = path_type.path.get_ident() {
                                        if "str" == ident.to_string() {
                                            return_type = Some(model::ReturnType::StaticStr);
                                        }
                                    }

                                    if return_type.is_none() {
                                        todo!("lookup the return reference type");
                                    }
                                },
                                _ => err!(trait_item.span(),
                                    "Unsupported return reference type: {}", ref_type.to_token_stream().to_string())
                            }
                        },
                        _ => todo!("unsupp"),
                    },
                }

                let return_type = return_type.ok_or(mkerr!(trait_item.span(), "Uable to parse return type!!"))?;
                let mut attribute_def = model::AttributeDefinition::from(return_type);

                let attrib = func.attrs.iter().find(|attrib| {
                    let last_path_segment = attrib.path().segments.last();
                    last_path_segment.is_some()
                        && ATTRIBUTE_HELPER_NAME != last_path_segment.unwrap().ident.to_string() 
                });

                if let Some(attrib) = attrib {
                    attrib.parse_nested_meta(|meta| {
                        let ident_name = meta.path.get_ident()
                            .ok_or(mkerr!(trait_item.span(), "Empty attribute"))?
                            .to_string();

                        let content;
                        syn::parenthesized!(content in meta.input);

                        match &mut attribute_def {
                            model::AttributeDefinition::StaticStr(ref strdef) => {
                                match ident_name.as_str() {
                                    "default" => strdef.default = Some(content.parse::<syn::LitStr>()?.value()),
                                    "format" => {}
                                }
                            },
                            model::AttributeDefinition::UnsignedSize(_) => todo!(),
                            model::AttributeDefinition::UnsignedInteger64(_) => todo!(),
                            model::AttributeDefinition::Integer64(_) => todo!(),
                            model::AttributeDefinition::Float64(_) => todo!(),
                            model::AttributeDefinition::UnsignedInteger32(_) => todo!(),
                            model::AttributeDefinition::Integer32(_) => todo!(),
                            model::AttributeDefinition::Float32(_) => todo!(),
                            model::AttributeDefinition::Byte(_) => todo!(),
                            model::AttributeDefinition::EnumVariant(_) => todo!(),
                            model::AttributeDefinition::Relation(_) => todo!(),
                        }

                        Ok(())
                    })?;
                }

                let method = model::Method::new(func.sig.ident.to_string(), return_type, attribute_def);
                methods.push(method);
                
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

    // remove all enumtrait helper attributes
    for trait_item in &mut item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                func.attrs.clear();  //TODO: only delete our stuff
            }
            _ => {}
        }
    }

    Ok(EnumTraitMacroOutput {
        tokens: item.to_token_stream(),
        model: model::EnumTrait::new(identifier, methods)
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

        dbg!(&model);
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
