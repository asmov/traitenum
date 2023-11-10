use quote::{self, ToTokens};
use syn::spanned::Spanned;
use syn::{self, Token};
use proc_macro2;
use bincode;

use crate::{model, ENUM_ATTRIBUTE_HELPER_NAME};
use crate::parse::{self, ParseAttribute};
use crate::{synerr, mksynerr, TRAIT_ATTRIBUTE_HELPER_NAME};

const ERROR_PREFIX: &'static str = "traitenum: ";
const MODEL_PREFIX: &'static str = "TRAITENUM_";

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
        synerr!(item.ident.span(),
            "Trait name does not match #enumtrait(<absolute trait path>): {}", identifier.name());
    }

    let mut methods: Vec<model::Method> = Vec::new(); 

    for trait_item in &item.items {
        let span = trait_item.span();

        match trait_item {
            syn::TraitItem::Fn(func) => {
                // ignore functions with default implementations
                if func.default.is_some() {
                    continue;
                }

                let method_name = func.sig.ident.to_string();
                let mut return_type: Option<model::ReturnType> = None;

                match &func.sig.output {
                    syn::ReturnType::Default => synerr!(span, "Default return types () are not supported"),
                    syn::ReturnType::Type(_, ref returntype) => match **returntype {
                        syn::Type::Path(ref path_type) => {
                            return_type = match model::ReturnType::try_from(&path_type.path) {
                                Ok(v) => Some(v),
                                Err(e) => synerr!(span, e)
                            };
                        },
                        syn::Type::Reference(ref ref_type) => {
                            // only elided and static lifetimes are supported
                            let _has_static_lifetime = match &ref_type.lifetime {
                                Some(lifetime) => {
                                    if "static" == lifetime.ident.to_string() {
                                        true
                                    } else {
                                        synerr!(span, "Only elided and static lifetimes are supported for return types")
                                    }
                                },
                                None => false
                            };

                            // mutability isn not supported
                            if ref_type.mutability.is_some() {
                                synerr!(span, "Mutable return types are not supported")
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
                                _ => synerr!(span,
                                    "Unsupported return reference type: {}", ref_type.to_token_stream().to_string())
                            }
                        },
                        _ => todo!("unsupp"),
                    },
                }

                // throw an error if the the wrong helper (enum vs trait) was used
                let attrib = func.attrs.iter().find(|attrib| {
                    attrib.path().segments.first().is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
                });

                if attrib.is_some() {
                    synerr!(span, "Wrong attribute helper was used for trait: `#[{}]`. Please use for `#[{}]` traits.",
                        ENUM_ATTRIBUTE_HELPER_NAME, TRAIT_ATTRIBUTE_HELPER_NAME);
                }

                let return_type = return_type.ok_or(mksynerr!(trait_item.span(), "Uable to parse return type!!"))?;

                // expected: traitenum::<AttributeDefinition>. we match against the 'traitenum' segment to get started.
                let attrib = func.attrs.iter().find(|attrib| {
                    attrib.path().segments.first().is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
                });

                let attribute_def = if let Some(attrib) = attrib {
                    parse::parse_attribute(attrib, trait_item.span(), return_type)?
                } else {
                    model::AttributeDefinition::from(return_type)
                };

                let method = model::Method::new(method_name, return_type, attribute_def);
                methods.push(method);
                
            },
            syn::TraitItem::Type(_) => {
                todo!("type")
            },
            syn::TraitItem::Macro(_) => {},
            syn::TraitItem::Verbatim(_) => {},
            syn::TraitItem::Const(_) => {},
            _ => () 
        }
    }

    // strip out all #enumtrait helper attributes
    for trait_item in &mut item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                let mut count = 0;  
                func.attrs.retain(|attrib| {
                    if attrib.path().segments.first() .is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string()) {
                        count += 1;
                        false
                    } else {
                        true
                    }
                });

                // we only process one attribute helper per method. curtail expectations with an error.
                if count > 1 {
                    synerr!(trait_item.span(), "Only one #traitenum helper attribute per method is supported");
                }
            
            },
            _ => ()
        }
    }

    // throw an error if the user uses "traitenum" instead of enumtrait

    Ok(EnumTraitMacroOutput {
        tokens: item.to_token_stream(),
        model: model::EnumTrait::new(identifier, methods)
    })
}

pub fn traitenum_macro(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
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

pub fn derive_traitenum_macro(
        item: proc_macro2::TokenStream,
        model_bytes: &'static [u8]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let model = model::EnumTrait::from(model_bytes);

    let item: syn::DeriveInput = syn::parse2(item)?;
    let span = item.span();
    let data_enum: &syn::DataEnum = match item.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => synerr!(span, "Only enums are supported for #[{}]", ENUM_ATTRIBUTE_HELPER_NAME)
    };

    let trait_ident = syn::Ident::new(model.identifier().name(), item.span());
    let item_ident = &item.ident;

    let method_outputs = model.methods().iter().map(|method| {
        let func: syn::Ident = syn::Ident::new(method.name(), span);
        let return_sig = match method.return_type() {
            model::ReturnType::StaticStr => quote::quote!{ &'static str },
            model::ReturnType::UnsignedSize => quote::quote!{ usize },
            model::ReturnType::UnsignedInteger64 => quote::quote!{ u64 },
            model::ReturnType::Integer64 => quote::quote!{ i64 },
            model::ReturnType::Float64 => quote::quote!{ f64 },
            model::ReturnType::UnsignedInteger32 => quote::quote!{ u32 },
            model::ReturnType::Integer32 => quote::quote!{ i32 },
            model::ReturnType::Float32 => quote::quote!{ f32 },
            model::ReturnType::Byte => quote::quote!{ u8 },
            model::ReturnType::Type => todo!("return sig: type"),
            model::ReturnType::TypeReference => todo!("return sig type reference"),
        };

        let variant_outputs = data_enum.variants.iter().map(|variant| {
            let variant_ident = &variant.ident;

            quote::quote!{
                Self::#variant_ident => todo!(),
            }
        });

        let output = quote::quote!{
            fn #func(&self) -> #return_sig {
                match self {
                    #(#variant_outputs)*
                }
            }
        };
        
        output
    });

    let output = quote::quote!{
        impl #trait_ident for #item_ident {
            #(#method_outputs)*
        }
    };

    Ok(output)
}

#[cfg(test)]
mod tests {
    use quote;

    use crate::TRAIT_ATTRIBUTE_HELPER_NAME;

    #[test]
    fn test_parse_enumtrait() {
        let attribute_src = quote::quote!{
            crate::tests::MyTrait
        };

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test default parsing
                fn name(&self) -> &'static str;
                #[enumtrait::Str(default(":)"))]
                fn emote(&self) -> &'static str;
                // test ordinal
                #[enumtrait::Num(default(44))]
                fn column(&self) -> usize;
                // test default implementation
                fn something_default(&self) {
                    todo!("done");
                }
            }
        };
        
        let model = super::parse_enumtrait(attribute_src, item_src).unwrap().model;

        assert_eq!(vec!["crate", "tests"], model.identifier().path());
        assert_eq!("MyTrait", model.identifier().name());

        dbg!(&model);
    }

    #[test]
    fn test_parse_enumtrait_errors() {
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
            "Empty #[{}(<pathspec>)] should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);
        
        // test error: mismatched trait name with identifier
        let attribute_src = quote::quote!{ crate::tests::TheirTrait };
        assert!(super::parse_enumtrait(attribute_src, simple_item_src.clone()).is_err(),
            "Mismatched trait name and #[{}(<pathspec>)] identifier should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);
    }

    #[test]
    fn test_traitenum_macro() {

    }
}
