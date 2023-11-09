use std::str::FromStr;

use quote::ToTokens;
use syn::{self, parse};

use crate::model;
use crate::{synerr, mksynerr, ATTRIBUTE_HELPER_NAME};

const ERROR_PREFIX: &'static str = "traitenum: ";

impl parse::Parse for model::Identifier {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut full_path: syn::Path = input.parse().map_err(|_| syn::Error::new(input.span(),
            "Unable to parse trait #enumtrait(<absolute trait path>)"))?;
        let name = full_path.segments.pop()
            .ok_or(syn::Error::new(input.span(), "enumtrait: Unable to parse trait name"))?
            .value().ident.to_string();
        let path = full_path.segments.pairs()
            .map(|pair| pair.value().ident.to_string())
            .collect();

        Ok(Self::new(path, name))
    }
}

impl TryFrom<&syn::Path> for model::ReturnType {
    type Error = String;

    fn try_from(path: &syn::Path) -> Result<Self, Self::Error> {
        let ident = match path.get_ident() {
            Some(v) => v.to_string(),
            None => return Err(format!("Unsupported return type: {}", path.to_token_stream().to_string()))
        };

        match ident.as_str() {
            "usize" => Ok(model::ReturnType::UnsignedSize),
            _ => Err(format!("Unsupported return type: {}", path.to_token_stream().to_string()))
        }
    }
}

pub trait ParseAttribute {
    fn parse_attribute(
            attr: &syn::Attribute,
            span: proc_macro2::Span,
            return_type: model::ReturnType) -> Result<(), syn::Error>;
}

pub trait ParseAttributeMeta {
    fn parse_attribute_meta(
            def: &mut model::AttributeDefinition,
            name: &str,
            content: syn::parse::ParseBuffer,
            span: proc_macro2::Span,
            return_type: model::ReturnType) -> Result<(), syn::Error>;
}


pub(crate) fn parse_attribute(
        attr: &syn::Attribute,
        span: proc_macro2::Span,
        return_type: model::ReturnType) -> Result<model::AttributeDefinition, syn::Error> {
    if attr.path().segments.len() != 2 {
        synerr!(span, "Unable to parse helper attribute: `{}`. Format: {}::DefinitionName",
            ATTRIBUTE_HELPER_NAME,
            attr.path().to_token_stream().to_string())
    }

    let attribute_def_name = attr.path().segments.last()
        .ok_or(mksynerr!(span,
            "Empty helper attribute definition name. Format: {}::DefinitionName",
            ATTRIBUTE_HELPER_NAME))?.ident.to_string();

    let mut def = model::AttributeDefinition::from(return_type);
    attr.parse_nested_meta(|meta| {
        let name = meta.path.get_ident()
            .ok_or(mksynerr!(span, "Unknown definition property: `{}`",
                meta.path.to_token_stream().to_string()))?
            .to_string();

        let content;
        syn::parenthesized!(content in meta.input);

        match attribute_def_name.as_str() {
            "Str" | "StaticStr" => 
                parse_string_attribute_meta(&mut def, &name, content, span, return_type)?,
            "Num" | "Number" => 
                parse_generic_number_attribute_meta(&mut def, &name, content, span, return_type)?,
                _ => synerr!(span, "Unknown attribute definition: {}", attribute_def_name)

        };

        Ok(())
    })?;

    Ok(def)
}

fn parse_string_attribute_meta( // item
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        return_type: model::ReturnType) -> Result<(), syn::Error>
{
    match def {
        model::AttributeDefinition::StaticStr(def) => parse_static_str_attribute_meta(def, name, content, span, return_type),
        _ => synerr!(span, "Unsupported def")
    }
}

fn parse_static_str_attribute_meta(
        def: &mut model::StaticStrAttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        return_type: model::ReturnType) -> Result<(), syn::Error> {
    match name {
       "default" => {
            def.default = Some(content.parse::<syn::LitStr>()?.value())
       },
       _ => synerr!(span, "Unknown attribute definition property: {}", name)
    }

    Ok(())
}

fn parse_generic_number_attribute_meta( // item
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        return_type: model::ReturnType) -> Result<(), syn::Error>
{
    match def {
        model::AttributeDefinition::UnsignedSize(def) => parse_number_attribute_meta(def, name, content, span, return_type),
        _ => synerr!(span, "Unsupported def")
    }
}
 
fn parse_number_attribute_meta<N>( // item
        def: &mut model::NumberAttributeDefinition<N>,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        return_type: model::ReturnType) -> Result<(), syn::Error>
where
    N: FromStr,
    N::Err: std::fmt::Display
{
    let is_float = match return_type {
        model::ReturnType::UnsignedSize => false,
        model::ReturnType::UnsignedInteger64 => false,
        model::ReturnType::Integer64 => false,
        model::ReturnType::Float64 => true,
        model::ReturnType::UnsignedInteger32 => false,
        model::ReturnType::Integer32 => false,
        model::ReturnType::Float32 => true,
        model::ReturnType::Byte => false,
        _ => synerr!(span, "Unexpected return type for number attribute: {:#?}", return_type)
    };

    macro_rules! parsenum {
        () => {
            if is_float {
                content.parse::<syn::LitFloat>()?.base10_parse()?
            } else {
                content.parse::<syn::LitInt>()?.base10_parse()?
            }
        };
    }
    
    //let x: N = content.parse::<syn::LitFloat>()?.base10_parse()?;

    match name {
       "default" => {
            let x: N = parsenum!();
            def.default = Some(x)
       },
       _ => synerr!(span, "Unknown attribute definition property: {}", name)
    }

    Ok(())
}