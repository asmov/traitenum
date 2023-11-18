use quote::{self, ToTokens};
use syn;
use proc_macro2;

use crate::{
    model, model::parse,
    synerr, mksynerr,
    ERROR_PREFIX, ENUM_ATTRIBUTE_HELPER_NAME};


#[derive(Debug)]
pub(crate) struct TraitEnumMacroOutput {
    pub(crate) tokens: proc_macro2::TokenStream,
    pub(crate) model: model::TraitEnum
}

pub fn traitenum_derive_macro(
        item: proc_macro2::TokenStream,
        model_bytes: &[u8]) -> Result<proc_macro2::TokenStream, syn::Error> {
    let TraitEnumMacroOutput { tokens, model: _model } = parse_traitenum_macro(item, model_bytes)?;
    Ok(tokens)
}
 
pub(crate) fn parse_traitenum_macro(
    item: proc_macro2::TokenStream,
    enumtrait_model_bytes: &[u8]) -> Result<TraitEnumMacroOutput, syn::Error>
{
    let enumtrait = model::EnumTrait::deserialize(enumtrait_model_bytes).unwrap();
    let input: syn::DeriveInput = syn::parse2(item)?;
    // the actual parsing is done with this call, the rest is building a tokenstream
    let traitenum = parse_traitenum_model(&input, &enumtrait)?;
    let trait_ident = syn::Ident::new(enumtrait.identifier().name(), proc_macro2::Span::call_site());
    let data_enum = data_enum(&input)?;

    // write a method for each one defined by the enum trait, which returns the value defined by each enum variant
    let method_outputs = enumtrait.methods().iter().map(|method| {
        let method_name = method.name();
        let func: syn::Ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
        let return_type = method.return_type_tokens();

        match method.attribute_definition() {
            model::AttributeDefinition::Relation(_) => {
                let relation_path: syn::Path = traitenum.relation_enum_identifier(method_name).unwrap().into();
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

        let type_ident = syn::Ident::new(associated_type.name(), proc_macro2::Span::call_site());
        let enum_ident = if relation_enum_id.path().is_empty() {
            syn::Path::from(relation_enum_id)
        } else {
            syn::Path::from(relation_enum_id.base().unwrap())
        };
        
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
        _ => synerr!("Only enums are supported for #[{}]", ENUM_ATTRIBUTE_HELPER_NAME)
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
                .ok_or(mksynerr!("Invalid traitenum attribute. It is not an ident token: `{}`",
                    meta.path.to_token_stream().to_string()))?
                .to_string();

            // prevent duplicates
            if traitenum_build.has_relation_enum(&attr_name) {
                synerr!("Duplicate traitenum attribute for enum: {}", attr_name);
            }

            // find the matching trait method by name
            let attribute_definition = enumtrait.methods().iter()
                .find(|m| { m.name() == attr_name })
                .ok_or_else(|| mksynerr!("No matching trait method for enum attribute: {}", attr_name))?
                .attribute_definition();

            // ensure that we're using a relation attribute definition for this method
            match attribute_definition {
                model::AttributeDefinition::Relation(_) => (),
                _ => synerr!("Trait method definition is not a Relation as expected for enum attribute: {}", attr_name)
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
            let definition = method.attribute_definition();
            if variant_build.has_value(method_name) {
                continue;
            } else if !definition.needs_value() {
                continue;
            } else if !definition.has_default_or_preset() {
                synerr!("Missing value for attribute `{}`: {}", method_name, variant_name);
            } else {
                let value = definition.default_or_preset(&variant_name, ordinal).unwrap();
                variant_build.value(method_name.to_string(), model::AttributeValue::new(value));
            }
        }

        // if this was a Rel attribute that needs a value, we create a relation_enum for it, as it wasn't
        // processed at the top of the enum
        /*for method in enumtrait.methods() {
            match method.attribute_definition() {
                model::AttributeDefinition::Relation(reldef) => {
                    match reldef.relationship {
                        Some(model::Relationship::OneToMany) => {
                            let method_name = method.name();
                            let attr_value = variant_build.get_value(&method_name).unwrap();
                            if let model::Value::Relation(id) = attr_value.value() {
                                traitenum_build.relation_enum(method_name.to_owned(), id.to_owned());
                            } else {
                                unreachable!();
                            }
                        },
                        Some(_) => (),
                        None => unreachable!(),
                    }
                },
                _ => ()
            } 
        }TODONE*/

        traitenum_build.variant(variant_build.build());
        ordinal += 1;
    }

    Ok(traitenum_build.build())
}

