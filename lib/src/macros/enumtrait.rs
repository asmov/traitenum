use quote::{self, ToTokens};
use syn::{self, spanned::Spanned};
use proc_macro2;
use bincode;

use crate::{
    model, macros, model::parse,
    synerr, mksynerr,
    ERROR_PREFIX, TRAIT_ATTRIBUTE_HELPER_NAME, ENUM_ATTRIBUTE_HELPER_NAME };


#[derive(Debug)]
pub(crate) struct EnumTraitMacroOutput {
    pub(crate) tokens: proc_macro2::TokenStream,
    pub(crate) model: model::EnumTrait
}

pub fn enumtrait_macro(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
        -> Result<proc_macro2::TokenStream, syn::Error> {
    let EnumTraitMacroOutput {tokens, model} = parse_enumtrait_macro(attr, item)?;
    let model_name = syn::Ident::new(
        &format!("{}{}", macros::MODEL_BYTES_NAME, model.identifier().name().to_uppercase()), proc_macro2::Span::call_site());

    let bytes = &bincode::serialize(&model).unwrap();
    let bytes_len = bytes.len();
    let bytes_literal = syn::LitByteStr::new(bytes, tokens.span());

    let output = quote::quote!{
        pub const #model_name: &'static [u8; #bytes_len] = #bytes_literal;

        #tokens
    };

    Ok(output)
}

pub(crate) fn parse_enumtrait_macro(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
        -> Result<EnumTraitMacroOutput, syn::Error> {
    let identifier: model::Identifier = syn::parse2(attr)?;
    
    let mut item: syn::ItemTrait = syn::parse2(item)?;
    if identifier.name() != item.ident.to_string() {
        synerr!("Trait name does not match #enumtrait(<absolute trait path>): {}", identifier.name());
    }

    let mut methods: Vec<model::Method> = Vec::new(); 
    let mut partial_types: Vec<model::AssociatedTypePartial> = Vec::new();
    let mut types: Vec<model::AssociatedType> = Vec::new();

    for trait_item in &item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                // ignore functions with default implementations
                if func.default.is_some() {
                    continue;
                }

                let method_name = func.sig.ident.to_string();
                let mut return_type: Option<model::ReturnType> = None;
                let mut return_type_identifier: Option<model::Identifier> = None;

                match &func.sig.output {
                    syn::ReturnType::Default => synerr!("Default return types () are not supported"),
                    syn::ReturnType::Type(_, ref returntype) => match **returntype {
                        syn::Type::Path(ref path_type) => {
                            match model::ReturnType::try_from(&path_type.path) {
                                Ok(v) => return_type = Some(v),
                                Err(_) => {
                                    return_type = Some(model::ReturnType::Type);
                                    return_type_identifier = match model::Identifier::try_from(&path_type.path) {
                                        Ok(id) => Some(id),
                                        Err(_) => synerr!("Unsupported return type: {}",
                                            &path_type.path.to_token_stream().to_string())
                                    }
                                }
                                                                };
                        },
                        syn::Type::Reference(ref ref_type) => {
                            // only elided and static lifetimes are supported
                            let _has_static_lifetime = match &ref_type.lifetime {
                                Some(lifetime) => {
                                    if "static" == lifetime.ident.to_string() {
                                        true
                                    } else {
                                        synerr!("Only elided and static lifetimes are supported for return types")
                                    }
                                },
                                None => false
                            };

                            // mutability isn not supported
                            if ref_type.mutability.is_some() {
                                synerr!("Mutable return types are not supported")
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
                                _ => synerr!("Unsupported return reference type: {}",
                                        ref_type.to_token_stream().to_string())
                            }
                        },
                        _ => todo!("Unimplemented trait return type"),
                    },
                }

                // throw an error if the the wrong helper (enum vs trait) was used
                let attrib = func.attrs.iter().find(|attrib| {
                    attrib.path().segments.first().is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
                });

                if attrib.is_some() {
                    synerr!("Wrong attribute helper was used for trait: `#[{}]`. Please use for `#[{}]` traits.",
                        ENUM_ATTRIBUTE_HELPER_NAME, TRAIT_ATTRIBUTE_HELPER_NAME);
                }

                let return_type = return_type.ok_or(mksynerr!("Uable to parse return type!!"))?;

                // expected: traitenum::<AttributeDefinition>. we match against the 'traitenum' segment to get started.
                let attrib = func.attrs.iter().find(|attrib| {
                    attrib.path().segments.first().is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
                });

                let attribute_def = if let Some(attrib) = attrib {
                    parse::parse_attribute_definition(attrib, return_type, return_type_identifier)?
                } else {
                    // build a default attribute definition for this method
                    model::AttributeDefinition::partial(None, return_type, return_type_identifier)
                        .map_err(|e| mksynerr!("Unable to parse definition from return signature for `{}` :: {}",
                            method_name, e))?
                };

                // ensure that the attribute definition adheres to its own rules
                if let Err(errmsg) = attribute_def.validate() {
                    synerr!(errmsg);
                }

                let method = model::Method::new(method_name, return_type, attribute_def);
                methods.push(method);
                
            },
            syn::TraitItem::Type(type_item) => {
                let type_identifier = model::Identifier::from(&type_item.ident);
                if type_item.bounds.len() != 1 {
                    synerr!("Unsupported trait bounds for associated type: {}", type_identifier.name())
                }

                let trait_identifier = match type_item.bounds.first().unwrap() {
                    syn::TypeParamBound::Trait(trait_bound) => model::Identifier::from(&trait_bound.path),
                    syn::TypeParamBound::Lifetime(_) => 
                        synerr!("Unsupported trait bounds for associated type: {}", type_identifier.name()),
                    syn::TypeParamBound::Verbatim(_) => 
                        synerr!("Unsupported trait bounds for associated type: {}", type_identifier.name()),
                    _ =>
                        synerr!("Unsupported trait bounds for associated type: {}", type_identifier.name())
                };

                let partial_associated_type = model::AssociatedTypePartial{
                    name: type_identifier.name().to_owned(),
                    trait_identifier,
                    matched: false
                };

                partial_types.push(partial_associated_type);
            },
            syn::TraitItem::Macro(_) => {},
            syn::TraitItem::Verbatim(_) => {},
            syn::TraitItem::Const(_) => {},
            _ => () 
        }
    }

    // match partially constructed associated types to relation methods and finalize
    for method in &methods {
        let relation_def = match method.attribute_definition() {
            model::AttributeDefinition::Relation(reldef) => reldef,
            _ => continue
        };

        let method_return_id = relation_def.identifier();
        let partial_type_result = partial_types.iter_mut()
            .filter(|t| !t.matched)
            .find(|t| t.name == method_return_id.name());

        let partial_type = match partial_type_result {
            Some(t) => t,
            None => synerr!("Unable to find a associated type for relationship definition: {}", method.name())
        };

        let associated_type = model::AssociatedType::new(
            partial_type.name.to_owned(),
            method.name().to_owned(),
            partial_type.trait_identifier.to_owned());

        types.push(associated_type);
        partial_type.matched = true;
    }

    // if there are remaining unmatched associated types, error out
    if let Some(unmatched_type) = partial_types.iter().find(|t| !t.matched) {
        synerr!("No matching relationship definition for associated type: {}", unmatched_type.name);
    }

    // strip out all #enumtrait helper attributes
    for trait_item in &mut item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                let mut count = 0;  
                func.attrs.retain(|attrib| {
                    if attrib.path().segments.first()
                            .is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string()) {
                        count += 1;
                        false
                    } else {
                        true
                    }
                });

                // we only process one attribute helper per method. curtail expectations with an error.
                if count > 1 {
                    synerr!("Only one #traitenum helper attribute per method is supported");
                }
            
            },
            _ => ()
        }
    }

    Ok(EnumTraitMacroOutput {
        tokens: item.to_token_stream(),
        model: model::EnumTrait::new(identifier, methods, types)
    })
}

