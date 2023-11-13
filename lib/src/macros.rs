use std::collections::HashMap;

use quote::{self, ToTokens};
use syn::{self, spanned::Spanned};
use proc_macro2;
use bincode;

use crate::{model, model::parse, synerr, mksynerr, TRAIT_ATTRIBUTE_HELPER_NAME, ENUM_ATTRIBUTE_HELPER_NAME};

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
                                Err(e) => synerr!(span, "Unsupported return type: {}",
                                    &path_type.path.to_token_stream().to_string())
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

                // ensure that the attribute definition adheres to its own rules
                if let Err(errmsg) = attribute_def.validate() {
                    synerr!(span, errmsg);
                }

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

#[derive(Debug)]
pub struct TraitEnumMacroOutput {
    tokens: proc_macro2::TokenStream,
    model: model::TraitEnum
}


pub fn traitenum_derive_macro(
        item: proc_macro2::TokenStream,
        model_bytes: &[u8]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let output: TraitEnumMacroOutput = parse_traitenum(item, model_bytes)?;
    Ok(output.tokens)
}
 
fn parse_traitenum(
        item: proc_macro2::TokenStream,
        model_bytes: &[u8]) -> Result<TraitEnumMacroOutput, syn::Error> {
    let model = model::EnumTrait::from(model_bytes);

    let item: syn::DeriveInput = syn::parse2(item)?;
    let span = item.span();
    let data_enum: &syn::DataEnum = match item.data {
        syn::Data::Enum(ref data_enum) => data_enum,
        _ => synerr!(span, "Only enums are supported for #[{}]", ENUM_ATTRIBUTE_HELPER_NAME)
    };

    let trait_ident = syn::Ident::new(model.identifier().name(), item.span());
    let item_ident = &item.ident;

    let mut trait_enum = model::TraitEnum::partial(model::Identifier::from(&item.ident));

    // parse enum attribute values, if provided
    let mut ordinal: usize = 0;
    for variant in &data_enum.variants {
        let variant_name = variant.ident.to_string();
        // find the #[traitenum] attribute or continue
        let attribute = variant.attrs.iter()
            .find(|a| a.path().segments.first()
                .is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string()));

        let mut variant = if let Some(attribute) = attribute {
            parse_variant(&variant_name, attribute, &model)?
        } else {
            model::Variant::partial(variant_name.to_owned())
        };

        // set attribute value defaults. throw errors where values are required, but not provided
        for method in model.methods() {
            let method_name = method.name();
            if variant.has(method_name) {
                continue;
            } else if !method.attribute_definition().has_default_or_preset() {
                synerr!(span, "Missing value for attribute `{}`: {}", method_name, variant_name);
            } else {
                let value = method.attribute_definition().default_or_preset(&variant_name, ordinal).unwrap();
                variant.set_value(method_name.to_string(), model::AttributeValue::new(value));

            }
        }

        trait_enum.push_variant(variant);
        ordinal += 1;
    }


    // write a method for each one defined by the enum trait, which returns the value defined by each enum variant
    let method_outputs = model.methods().iter().map(|method| {
        let method_name = method.name();
        let func: syn::Ident = syn::Ident::new(method_name, span);
        let return_type = method.return_type().to_token_stream();

        // create the match{} body of the method, mapping variants to their return value
        let variant_outputs = data_enum.variants.iter().map(|variant_data| {
            let variant_ident = &variant_data.ident;
            let variant_name = variant_ident.to_string();
            let value = trait_enum
                .variant(&variant_name).unwrap()
                .value(method_name).unwrap()
                .to_token_stream();

            quote::quote!{
                Self::#variant_ident => #value,
            }
        });

        // the final method signature and body
        let output = quote::quote!{
            fn #func(&self) -> #return_type {
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

    Ok(TraitEnumMacroOutput {
        tokens: output,
        model: trait_enum
    })
}

fn parse_variant(variant_name: &str, attr: &syn::Attribute, model: &model::EnumTrait)
        -> Result<model::Variant, syn::Error> {
    let mut variant = model::Variant::partial(variant_name.to_owned());
    attr.parse_nested_meta(|meta| {
        let attr_name = meta.path.get_ident()
            .ok_or(mksynerr!(attr.span(), "Invalid enum attribute: `{}`",
                meta.path.to_token_stream().to_string()))?
            .to_string();

        if variant.has(&attr_name) {
            synerr!(attr.span(), "Duplicate enum attribute value for: {}", attr_name);
        }

        let method = model.methods().iter().find(|m| m.name() == attr_name)
            .ok_or(mksynerr!(attr.span(), "Unknown enum attribute: {}", attr_name))?;

        let attribute_def = &method.attribute_definition();

        let content;
        syn::parenthesized!(content in meta.input);

        let value = match attribute_def {
            model::AttributeDefinition::Bool(_) => model::Value::Bool(
                content.parse::<syn::LitBool>()?.value()),
            model::AttributeDefinition::StaticStr(_) => model::Value::StaticStr(
                content.parse::<syn::LitStr>()?.value()),
            model::AttributeDefinition::UnsignedSize(_) => model::Value::UnsignedSize(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::AttributeDefinition::UnsignedInteger64(_) => todo!(),
            model::AttributeDefinition::Integer64(_) => todo!(),
            model::AttributeDefinition::Float64(_) => todo!(),
            model::AttributeDefinition::UnsignedInteger32(_) => todo!(),
            model::AttributeDefinition::Integer32(_) => todo!(),
            model::AttributeDefinition::Float32(_) => todo!(),
            model::AttributeDefinition::Byte(_) => todo!(),
            model::AttributeDefinition::Type(_) => todo!(),
            model::AttributeDefinition::Relation(_) => todo!(),
        };

        let attribute_value = model::AttributeValue::new(value);
        variant.set_value(attr_name, attribute_value);

        Ok(())
    })?;

    Ok(variant)
}

#[cfg(test)]
mod tests {
    use quote;

    use crate::{TRAIT_ATTRIBUTE_HELPER_NAME, model};

    #[test]
    fn test_parse_enumtrait() {
        let attribute_src = quote::quote!{
            crate::tests::MyTrait
        };

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test preset variant parsing
                #[enumtrait::Str(preset(Variant))]
                fn name(&self) -> &'static str;
                // test default parsing
                #[enumtrait::Str(default(":)"))]
                fn emote(&self) -> &'static str;
                // test ordinal
                #[enumtrait::Num(default(44))]
                fn column(&self) -> usize;
                #[enumtrait::Num(preset(Serial), start(3), increment(2))]
                fn serial(&self) -> u64;
                #[enumtrait::Bool(default(true))]
                fn able(&self) -> bool;
                // test default implementation
                fn something_default(&self) {
                    todo!("done");
                }
            }
        };
        
        let model = super::parse_enumtrait(attribute_src, item_src).unwrap().model;
        dbg!(&model);

        assert_eq!(vec!["crate", "tests"], model.identifier().path());
        assert_eq!("MyTrait", model.identifier().name());

        let item_src = quote::quote!{
            enum MyEnum {
                One,
                #[traitenum(name("2"))]
                Two,
                #[traitenum(able(false))]
                Three
            }
        };

        let model_bytes = bincode::serialize(&model).unwrap();
        let enum_model = super::parse_traitenum(item_src, &model_bytes).unwrap().model;
        dbg!(&enum_model);

        macro_rules! assert_variant_val {
            ($variant_name:literal, $attribute_name:literal, $value_type:ident, $expected:expr) => {
                match enum_model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                    model::Value::$value_type(ref val) => assert_eq!($expected, *val),
                    _ => assert!(false, "Incorrect value type for attribute: $attribute_name")
                }
            };
        }

        assert_variant_val!("One", "name", StaticStr, "One");
        assert_variant_val!("Two", "column", UnsignedSize, 44);
        assert_variant_val!("Two", "name", StaticStr, "2");
        assert_variant_val!("Three", "serial", UnsignedInteger64, 7);
        assert_variant_val!("Three", "able", Bool, false);
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
