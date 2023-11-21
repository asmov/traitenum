use quote::{self, ToTokens};
use syn::{self, spanned::Spanned};
use proc_macro2;

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

    let bytes = model.serialize().unwrap();
    let bytes_len = bytes.len();
    let bytes_literal = syn::LitByteStr::new(&bytes, tokens.span());

    let output = quote::quote!{
        pub const #model_name: &'static [u8; #bytes_len] = #bytes_literal;

        #tokens
    };

    Ok(output)
}

pub(crate) fn parse_enumtrait_macro(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream) -> syn::Result<EnumTraitMacroOutput>
{
    let identifier: model::Identifier = syn::parse2(attr)?;
    let mut trait_input: syn::ItemTrait = syn::parse2(item)?;

    if identifier.name() != trait_input.ident.to_string() {
        synerr!("Trait name does not match #enumtrait(<absolute trait path>): {}", identifier.name());
    }

    let mut methods: Vec<model::Method> = Vec::new(); 
    let mut partial_types: Vec<model::AssociatedTypePartial> = Vec::new();

    // We only support trait methods and trait types. Everything else is either ignored or denied
    for trait_item in &trait_input.items {
        match trait_item {
            // Build a model Method
            syn::TraitItem::Fn(func) => parse_trait_fn(&mut methods, func)?,
            // Partially build a model AssociatedType. We'll finalize with a second pass after the loop.
            syn::TraitItem::Type(type_item) => parse_trait_type(&mut partial_types, type_item)?,
            _ => ()
        }
    }

    // These were partially constructed using parse_trait_type(). Now we finalize construction with a second pass.
    // They will be used by the enum derive macro to build each traitenum's associated types. E.g., type Foo = Bar;
    let types: Vec<model::AssociatedType> = build_associated_types(&mut partial_types, &methods)?;

    // Remove all #[tratienum] attributes from the TokenStream now that we're done parsing them
    clean_helper_attributes(&mut trait_input)?;

    Ok(EnumTraitMacroOutput {
        tokens: trait_input.to_token_stream(),
        model: model::EnumTrait::new(identifier, methods, types)
    })
}

fn parse_trait_fn(methods: &mut Vec<model::Method>, func: &syn::TraitItemFn) -> syn::Result<()> {
    // ignore functions with default implementations
    if func.default.is_some() {
        return Ok(());
    }

    let method_name = func.sig.ident.to_string();
    let (return_type, return_type_identifier) = parse_trait_fn_return(func)?;

    // Throw an error if the the wrong helper (enum vs trait) was used
    // 1. search for the wrong name ...
    let attrib = func.attrs.iter().find(|attrib| {
        attrib.path().segments.first().is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
    });

    // 2. throw the error
    if attrib.is_some() {
        synerr!("Wrong attribute helper was used for trait: `#[{}]`. Please use for `#[{}]` traits.",
            ENUM_ATTRIBUTE_HELPER_NAME, TRAIT_ATTRIBUTE_HELPER_NAME);
    }

    // We expect a helper attribute that defines each trait method.
    // E.g., #[traitenum::Str(preset(Variant))
    //     Where "::Str" refers to model::StrAttributeDefinition(def) and it's associated definition property "preset"
    // 1. match against the 'traitenum' path segment to get started.
    let attrib = func.attrs.iter().find(|attrib| {
        attrib.path().segments.first().is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
    });

    // Parse the attribute definition that is found. If not found, attempt to build a default based on method signature.
    let attribute_def = if let Some(attrib) = attrib {
        parse::parse_attribute_definition(attrib, return_type, return_type_identifier)?
    } else {
        model::AttributeDefinition::partial(None, return_type, return_type_identifier)
            .map_err(|e| mksynerr!("Unable to parse definition from return signature for `{}` :: {}", method_name, e))?
    };

    // Now perform a validation pass on all attribute definitions to enforce each def's specific rules
    if let Err(errmsg) = attribute_def.validate() {
        synerr!(errmsg);
    }

    let method = model::Method::new(method_name, return_type, attribute_def);
    methods.push(method);

    Ok(())
}

fn parse_trait_fn_return(func: &syn::TraitItemFn) -> syn::Result<(model::ReturnType, Option<model::Identifier>)> {
    let mut return_type: Option<model::ReturnType> = None;
    let mut return_type_identifier: Option<model::Identifier> = None;

    match &func.sig.output {
        syn::ReturnType::Default => synerr!("Default return types () are not supported"),
        syn::ReturnType::Type(_, ref returntype) => match **returntype {
            syn::Type::Path(ref path_type) => {
                // This will work for the primitize types that we support (usize, f32, bool, etc.).
                match model::ReturnType::try_from(&path_type.path) {
                    Ok(v) => return_type = Some(v),
                    // We model anything else as a ReturnType::Type. We convert the type's path to a model Identifier.
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
            // As for reference return types, we only support &'static str at the moment.
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

                // mutability is not supported
                if ref_type.mutability.is_some() {
                    synerr!("Mutable return types are not supported")
                }

                // basically just ensure that the ident is a "str"
                if let syn::Type::Path(ref path_type) = *ref_type.elem {
                    if let Some(ident) = path_type.path.get_ident() {
                        if "str" == ident.to_string() {
                            return_type = Some(model::ReturnType::StaticStr);
                        }
                    }
                }

                // ... the else statement for each nested if statement above
                if return_type.is_none() {
                    synerr!("Unsupported return reference type: {}", ref_type.to_token_stream().to_string())
                }
            },
            _ => synerr!("Unimplemented trait return type"),
        }
    }

    let return_type = return_type.expect("Unable to parse return type");
    Ok((return_type, return_type_identifier))
}

fn parse_trait_type(
    partial_types: &mut Vec<model::AssociatedTypePartial>,
    type_item: &syn::TraitItemType) -> syn::Result<()>
{
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

    Ok(())
}

fn clean_helper_attributes(trait_input: &mut syn::ItemTrait) -> syn::Result<()> {
    // strip out all #enumtrait helper attributes
    for trait_item in &mut trait_input.items {
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

    Ok(())
}

fn build_associated_types(
    partial_types: &mut Vec<model::AssociatedTypePartial>,
    methods: &Vec<model::Method>) -> syn::Result<Vec<model::AssociatedType>>
{
    let mut types: Vec<model::AssociatedType> = Vec::new();

    // match partially constructed associated types to relation methods and finalize
    for method in methods {
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
            partial_type.trait_identifier.to_owned(),
            *relation_def.nature.as_ref().unwrap());

        types.push(associated_type);
        partial_type.matched = true;
    }

    // if there are remaining unmatched associated types, error out
    if let Some(unmatched_type) = partial_types.iter().find(|t| !t.matched) {
        synerr!("No matching relationship definition for associated type: {}", unmatched_type.name);
    }

    Ok(types)
}

