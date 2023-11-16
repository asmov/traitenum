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
    let EnumTraitMacroOutput {tokens, model} = parse_enumtrait_macro(attr, item)?;
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

fn parse_enumtrait_macro(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
        -> Result<EnumTraitMacroOutput, syn::Error> {
    let identifier: model::Identifier = syn::parse2(attr)?;
    
    let mut item: syn::ItemTrait = syn::parse2(item)?;
    if identifier.name() != item.ident.to_string() {
        synerr!(item.ident.span(),
            "Trait name does not match #enumtrait(<absolute trait path>): {}", identifier.name());
    }

    let mut methods: Vec<model::Method> = Vec::new(); 
    let mut partial_types: Vec<model::AssociatedTypePartial> = Vec::new();
    let mut types: Vec<model::AssociatedType> = Vec::new();

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
                let mut return_type_identifier: Option<model::Identifier> = None;

                match &func.sig.output {
                    syn::ReturnType::Default => synerr!(span, "Default return types () are not supported"),
                    syn::ReturnType::Type(_, ref returntype) => match **returntype {
                        syn::Type::Path(ref path_type) => {
                            match model::ReturnType::try_from(&path_type.path) {
                                Ok(v) => return_type = Some(v),
                                Err(_) => {
                                    return_type = Some(model::ReturnType::Type);
                                    return_type_identifier = match model::Identifier::try_from(&path_type.path) {
                                        Ok(id) => Some(id),
                                        Err(_) => synerr!(span, "Unsupported return type: {}",
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
                        _ => todo!("Unimplemented trait return type"),
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
                    parse::parse_attribute_definition(attrib, trait_item.span(), return_type, return_type_identifier)?
                } else {
                    // build a default attribute definition for this method
                    model::AttributeDefinition::partial(None, return_type, return_type_identifier)
                        .map_err(|e| mksynerr!(span, "Unable to parse definition from return signature for `{}` :: {}",
                            method_name, e))?
                };

                // ensure that the attribute definition adheres to its own rules
                if let Err(errmsg) = attribute_def.validate() {
                    synerr!(span, errmsg);
                }

                let method = model::Method::new(method_name, return_type, attribute_def);
                methods.push(method);
                
            },
            syn::TraitItem::Type(type_item) => {
                let type_identifier = model::Identifier::from(&type_item.ident);
                if type_item.bounds.len() != 1 {
                    synerr!(span, "Unsupported trait bounds for associated type: {}", type_identifier.name())
                }

                let trait_identifier = match type_item.bounds.first().unwrap() {
                    syn::TypeParamBound::Trait(trait_bound) => model::Identifier::from(&trait_bound.path),
                    syn::TypeParamBound::Lifetime(_) => 
                        synerr!(span, "Unsupported trait bounds for associated type: {}", type_identifier.name()),
                    syn::TypeParamBound::Verbatim(_) => 
                        synerr!(span, "Unsupported trait bounds for associated type: {}", type_identifier.name()),
                    _ =>
                        synerr!(span, "Unsupported trait bounds for associated type: {}", type_identifier.name())
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
            None => synerr!(item.span(),
                "Unable to find a associated type for relationship definition: {}", method.name())
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
        synerr!(item.span(), "No matching relationship definition for associated type: {}", unmatched_type.name);
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
        model: model::EnumTrait::new(identifier, methods, types)
    })
}

#[derive(Debug)]
struct TraitEnumMacroOutput {
    tokens: proc_macro2::TokenStream,
    model: model::TraitEnum
}

pub fn traitenum_derive_macro(
        item: proc_macro2::TokenStream,
        model_bytes: &[u8]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let TraitEnumMacroOutput { tokens, model: _model } = parse_traitenum_macro(item, model_bytes)?;
    Ok(tokens)
}
 
fn parse_traitenum_macro(
        item: proc_macro2::TokenStream,
        enumtrait_model_bytes: &[u8]) -> Result<TraitEnumMacroOutput, syn::Error> {
    let enumtrait = model::EnumTrait::from(enumtrait_model_bytes);
    let input: syn::DeriveInput = syn::parse2(item)?;
    // the actual parsing is done with this call, the rest is building a tokenstream
    let traitenum = parse_traitenum_model(&input, &enumtrait)?;
    let trait_ident = syn::Ident::new(enumtrait.identifier().name(), input.span());
    let data_enum = data_enum(&input)?;

    // write a method for each one defined by the enum trait, which returns the value defined by each enum variant
    let method_outputs = enumtrait.methods().iter().map(|method| {
        let method_name = method.name();
        let func: syn::Ident = syn::Ident::new(method_name, input.span());
        let return_type = method.return_type_tokens();

        match method.attribute_definition() {
            model::AttributeDefinition::Relation(_) => {
                let relation_path = traitenum.relation_enum_identifier(method_name).unwrap().to_path(&input);
                return quote::quote!{
                    fn #func(&self) -> #return_type {
                        #relation_path
                    }
                }
            },
            _ => ()
        }

        // create the match{} body of the method, mapping variants to their return value
        let variant_outputs = data_enum.variants.iter().map(|variant_data| {
            let variant_ident = &variant_data.ident;
            let variant_name = variant_ident.to_string();
            let value = traitenum
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

    // define an associated type for each of the traitenum's relations
    // e.g., enum Foo { type OtherTraitEnum = OtherEnum; ... }
    let type_outputs = traitenum.relation_enums().map(|(relation_name, relation_enum_id)| {
        // all of the errors should have been handled during model parsing, so we panic here instead of Err 
        // fetch the attribute definition with the same name as the relation's name 
        let attribute_definition = enumtrait.methods().iter()
            .find(|m| { m.name() == relation_name })
            .expect(&format!("No matching relation definition for enum relation: {}", relation_name))
            .attribute_definition();
        // grab the associated relation definition for the attribute, which contains its Self::<Type> Identifier
        match attribute_definition {
            model::AttributeDefinition::Relation(_) => {},
            _ => unreachable!("Mismatched AttributeDefinition variant for traitenum relation: {}", relation_name)
        };

        let associated_type = enumtrait.types().iter().find(|t| t.relation_name() == relation_name)
            .expect(&format!("No matching associated type for enum relation: {}", relation_name));

        let type_ident = syn::Ident::new(associated_type.name(), input.span());
        let enum_ident = relation_enum_id.base().unwrap().to_path(&input);
        
        quote::quote!{
            type #type_ident = #enum_ident;
        }
    });

    let input_ident = &input.ident;

    let output = quote::quote!{
        impl #trait_ident for #input_ident {
            #(#type_outputs)*
            #(#method_outputs)*
        }
    };

    Ok(TraitEnumMacroOutput {
        tokens: output,
        model: traitenum
    })
}

fn data_enum(input: &syn::DeriveInput) -> Result<&syn::DataEnum, syn::Error> {
    match input.data {
        syn::Data::Enum(ref data_enum) => Ok(data_enum),
        _ => synerr!(input.span(), "Only enums are supported for #[{}]", ENUM_ATTRIBUTE_HELPER_NAME)
    }
}

fn parse_traitenum_model(input: &syn::DeriveInput, enumtrait: &model::EnumTrait)
        -> Result<model::TraitEnum, syn::Error> {
    let mut traitenum_build = model::TraitEnumBuilder::new();
    traitenum_build.identifier(model::Identifier::from(&input.ident));

    //parse top-level attributes (item.attr) as relations -> #[traitenum(<relation name>(<trait path>))]
    for attr in &input.attrs {
        attr.parse_nested_meta(|meta| {
            // this will be the method and relation name as well
            let attr_name = meta.path.get_ident()
                .ok_or(mksynerr!( input.span(),
                    "Invalid traitenum attribute. It is not an ident token: `{}`",
                    meta.path.to_token_stream().to_string()))?
                .to_string();

            // prevent duplicates
            if traitenum_build.has_relation_enum(&attr_name) {
                synerr!(input.span(), "Duplicate traitenum attribute for enum: {}", attr_name);
            }

            // find the matching trait method by name
            let attribute_definition = enumtrait.methods().iter()
                .find(|m| { m.name() == attr_name })
                .ok_or_else(|| mksynerr!(input.span(),
                    "No matching trait method for enum attribute: {}", attr_name))?
                .attribute_definition();

            // ensure that we're using a relation attribute definition for this method
            match attribute_definition {
                model::AttributeDefinition::Relation(_) => (),
                _ => synerr!(input.span(),
                    "Trait method definition is not a Relation as expected for enum attribute: {}", attr_name)
            }

            let content;
            syn::parenthesized!(content in meta.input);
            let relation_path: syn::Path = content.parse()?;
            traitenum_build.relation_enum(attr_name, relation_path.into());

            Ok(())
        })?;
    }


    // parse enum attribute values, if provided
    let data_enum = data_enum(input)?;
    let mut ordinal: usize = 0;
    for variant in &data_enum.variants {
        let variant_name = variant.ident.to_string();
        // find the #[traitenum] attribute or continue
        let attribute = variant.attrs.iter()
            .find(|a| a.path().segments.first()
                .is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string()));

        let mut variant_build = if let Some(attribute) = attribute {
            parse::parse_variant(&variant_name, attribute, &enumtrait)?
        } else {
            let mut build = model::VariantBuilder::new();
            build.name(variant_name.to_owned());
            build
        };

        // set attribute value defaults. throw errors where values are required, but not provided
        for method in enumtrait.methods() {
            let method_name = method.name();
            if variant_build.has_value(method_name) {
                continue;
            } else if !method.attribute_definition().needs_value() {
                continue;
            } else if !method.attribute_definition().has_default_or_preset() {
                synerr!(input.span(), "Missing value for attribute `{}`: {}", method_name, variant_name);
            } else {
                let value = method.attribute_definition().default_or_preset(&variant_name, ordinal).unwrap();
                variant_build.value(method_name.to_string(), model::AttributeValue::new(value));
            }
        }

        traitenum_build.variant(variant_build.build());
        ordinal += 1;
    }

    Ok(traitenum_build.build())
}

#[cfg(test)]
mod tests {
    use quote;
    use crate::{TRAIT_ATTRIBUTE_HELPER_NAME, model};

    /// Asserts that the expected value has been defined for a given enum variant
    macro_rules! assert_traitenum_value {
        ($model:ident, $variant_name:literal, $attribute_name:literal, $value_type:ident, $expected:expr) => {
            assert!($model.variant($variant_name).is_some(), "Variant doesn't exist: {}", $variant_name);
            assert!($model.variant($variant_name).unwrap().value($attribute_name).is_some(),
                "Variant attribute doesn't exist: {} -> {}", $variant_name, $attribute_name);
            match $model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                model::Value::$value_type(ref val) => assert_eq!($expected, *val),
                _ => assert!(false, "Incorrect value type for attribute: {}", $attribute_name)
            }
        };
    }

    /// Asserts that the expected enum value has been defined for a given enum variant
    macro_rules! assert_traitenum_value_enum {
        ($model:ident, $variant_name:literal, $attribute_name:literal, $expected:literal) => {
            match $model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                model::Value::EnumVariant(ref val) => assert_eq!($expected, val.to_string()),
                _ => assert!(false, "Incorrect value type for attribute: {}", $attribute_name)
            }
        };
    }

    #[test]
    fn test_parse_enumtrait() {
        let attribute_src = quote::quote!{
            crate::tests::MyTrait
        };

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test Rel many-to-one
                type ManyToOneEnum: OtherTrait;

                // test Str default
                #[enumtrait::Str(default(":)"))]
                fn str_default(&self) -> &'static str;
                // test Num default
                #[enumtrait::Num(default(44))]
                fn num_default(&self) -> usize;
               // test Bool default
                #[enumtrait::Bool(default(true))]
                fn bool_default(&self) -> bool;
                // test Enum default
                #[enumtrait::Enum(default(RPS::Rock))]
                fn enum_default(&self) -> RPS;
                // test Str variant preset
                #[enumtrait::Str(preset(Variant))]
                fn str_preset_variant(&self) -> &'static str;
                // test Num serial preset w/start and increment 
                #[enumtrait::Num(preset(Serial), start(3), increment(2))]
                fn num_preset_serial_all(&self) -> u64;
                // test Rel many-to-one
                #[enumtrait::Rel(relationship(ManyToOne))]
                fn many_to_one(&self) -> Self::ManyToOneEnum;
                // test default implementation
                fn default_implementation(&self) {
                    todo!();
                }
            }
        };
        
        let model = super::parse_enumtrait_macro(attribute_src, item_src).unwrap().model;
        dbg!(&model);

        assert_eq!(vec!["crate", "tests"], model.identifier().path());
        assert_eq!("MyTrait", model.identifier().name());

        let item_src = quote::quote!{
            #[traitenum(many_to_one(OtherEnum::My))]
            enum MyEnum {
                One,
                #[traitenum(str_preset_variant("2"))]
                Two,
                #[traitenum(bool_default(false))]
                Three,
                #[traitenum(enum_default(RPS::Scissors))]
                Four,
            }
        };

        let model_bytes = bincode::serialize(&model).unwrap();
        let super::TraitEnumMacroOutput {model: enum_model, tokens: enum_tokens} = super::parse_traitenum_macro(
            item_src, &model_bytes).unwrap();

        dbg!(&enum_model);
        dbg!(&enum_tokens.to_string());

        assert_traitenum_value!(enum_model, "One", "str_default", StaticStr, ":)");
        assert_traitenum_value!(enum_model, "Two", "num_default", UnsignedSize, 44);
        assert_traitenum_value!(enum_model, "Two", "str_preset_variant", StaticStr, "2");
        assert_traitenum_value!(enum_model, "Three", "num_preset_serial_all", UnsignedInteger64, 7);
        assert_traitenum_value_enum!(enum_model, "Three", "enum_default", "RPS::Rock");
        assert_traitenum_value_enum!(enum_model, "Four", "enum_default", "RPS::Scissors");
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
        assert!(super::parse_enumtrait_macro(attribute_src, simple_item_src.clone()).is_err(),
            "Empty #[{}(<pathspec>)] should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);
        
        // test error: mismatched trait name with identifier
        let attribute_src = quote::quote!{ crate::tests::TheirTrait };
        assert!(super::parse_enumtrait_macro(attribute_src, simple_item_src.clone()).is_err(),
            "Mismatched trait name and #[{}(<pathspec>)] identifier should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);
    }

    #[test]
    fn test_traitenum_macro() {

    }
}
