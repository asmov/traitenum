use std::str::FromStr;

use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{self, parse};

use crate::model;
use crate::{synerr, mksynerr, TRAIT_ATTRIBUTE_HELPER_NAME};

const ERROR_PREFIX: &'static str = "traitenum: ";

impl parse::Parse for model::Identifier {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let path: syn::Path = input.parse().map_err(|_| syn::Error::new(input.span(),
            "Unable to parse trait #enumtrait(<absolute trait path>)"))?;
        Self::try_from(&path)
            .or_else(|_| synerr!(input.span(), "Unable to parse Identifier from Path: {}",
                path.to_token_stream().to_string()))
    }
}

impl From<&syn::Ident> for model::Identifier{
    fn from(ident: &syn::Ident) -> Self {
        model::Identifier::new(Vec::new(), ident.to_string())
    }
}

impl From<&syn::Path> for model::Identifier{
    fn from(path: &syn::Path) -> Self {
        let mut path = path.clone();
        let name = path.segments.pop().unwrap()
            .value().ident.to_string();
        let path = path.segments.pairs()
            .map(|pair| pair.value().ident.to_string())
            .collect();

        Self::new(path, name)
    }
}

impl From<syn::Path> for model::Identifier{
    fn from(path: syn::Path) -> Self {
        let mut path = path;
        let name = path.segments.pop().unwrap()
            .value().ident.to_string();
        let path = path.segments.pairs()
            .map(|pair| pair.value().ident.to_string())
            .collect();

        Self::new(path, name)
    }
}


impl TryFrom<&syn::Path> for model::ReturnType {
    type Error = ();

    fn try_from(path: &syn::Path) -> Result<Self, Self::Error> {
        match path.get_ident() {
            Some(v) => model::ReturnType::from_str(&v.to_string()),
            None => Err(()) 
        }
    }
}

pub(crate) fn parse_attribute_definition(
        attr: &syn::Attribute,
        span: proc_macro2::Span,
        return_type: model::ReturnType,
        return_type_id: Option<model::Identifier>
    ) -> Result<model::AttributeDefinition, syn::Error> {
    if attr.path().segments.len() != 2 {
        synerr!(span, "Unable to parse helper attribute: `{}`. Format: {}::DefinitionName",
            TRAIT_ATTRIBUTE_HELPER_NAME,
            attr.path().to_token_stream().to_string())
    }

    let definition_name = attr.path().segments.last()
        .ok_or(mksynerr!(span,
            "Empty helper attribute definition name. Format: {}::DefinitionName",
            TRAIT_ATTRIBUTE_HELPER_NAME))?.ident.to_string();

    let mut def = model::AttributeDefinition::partial(Some(&definition_name), return_type, return_type_id)
        .map_err(|e| mksynerr!(span, "Unable to parse return type for definition `{}` :: {}",
            attr.path().to_token_stream().to_string(), e))?;

    attr.parse_nested_meta(|meta| {
        let name = meta.path.get_ident()
            .ok_or(mksynerr!(span, "Unknown definition property: `{}`",
                meta.path.to_token_stream().to_string()))?
            .to_string();

        let content;
        syn::parenthesized!(content in meta.input);

        match definition_name.as_str() {
            model::BoolAttributeDefinition::DEFINITION_NAME =>  
                parse_bool_attribute_definition(&mut def, &name, content, span, return_type)?,
            model::StaticStrAttributeDefinition::DEFINITION_NAME => 
                parse_string_attribute_definition(&mut def, &name, content, span, return_type)?,
            model::NumberAttributeDefinition::<usize>::DEFINITION_NAME => 
                parse_number_attribute_definition(&mut def, &name, content, span, return_type)?,
            model::FieldlessEnumAttributeDefinition::DEFINITION_NAME =>
                parse_enum_attribute_definition(&mut def, &name, content, span, return_type)?,
            model::RelationAttributeDefinition::DEFINITION_NAME =>
                parse_relation_attribute_definition(&mut def, &name, content, span, return_type)?,
             _ => synerr!(span, "Unknown attribute definition: {}", definition_name)

        };

        Ok(())
    })?;

    Ok(def)
}

const DEFINITION_DEFAULT: &'static str = "default";
const DEFINITION_PRESET: &'static str = "preset";

fn parse_bool_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        _return_type: model::ReturnType) -> Result<(), syn::Error> {
    let booldef = match def {
        model::AttributeDefinition::Bool(def) => def,
        _ => synerr!(span, "Incorrect attribute definition for return type for property: {}", name)
    };

    match name {
       DEFINITION_DEFAULT => {
            booldef.default = Some(content.parse::<syn::LitBool>()?.value())
       },
       _ => synerr!(span, "Unknown attribute definition property: {}", name)
    }

    Ok(())
}

fn parse_enum_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        _return_type: model::ReturnType) -> Result<(), syn::Error> {
    let enumdef = match def {
        model::AttributeDefinition::FieldlessEnum(def) => def,
        _ => synerr!(span, "Mismatched definition for Enum: {}", name)
    };

    match name {
       DEFINITION_DEFAULT => {
            let id: model::Identifier = content.parse()?;
            enumdef.default = Some(id)
       },
       _ => synerr!(span, "Unknown definition property for Enum: {}", name)
    }

    Ok(())
}

fn parse_string_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        _return_type: model::ReturnType) -> Result<(), syn::Error> {
    let strdef = match def {
        model::AttributeDefinition::StaticStr(def) => def,
        _ => synerr!(span, "Mismatched definition for Str: {}", name)
    };

    match name {
       DEFINITION_DEFAULT => {
            strdef.default = Some(content.parse::<syn::LitStr>()?.value())
       },
       DEFINITION_PRESET => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let preset = model::StringPreset::from_str(&variant_name)
                .or(Err(mksynerr!(span, "Unknown String preset: {}", variant_name)))?;
            strdef.preset = Some(preset);
       },
       _ => synerr!(span, "Unknown definition property for Str: {}", name)
    }

    Ok(())
}

const DEFINITION_RELATIONSHIP: &'static str = "relationship";

fn parse_relation_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        _return_type: model::ReturnType) -> Result<(), syn::Error> {
    let reldef = match def {
        model::AttributeDefinition::Relation(def) => def,
        _ => unreachable!("Unexpected definition type: {}", model::RelationAttributeDefinition::DEFINITION_NAME)
    };

    match name {
       DEFINITION_RELATIONSHIP => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let relationship = model::Relationship::from_str(&variant_name)
                .or(Err(mksynerr!(span, "Unknown relationship: {}", variant_name)))?;
            reldef.relationship = Some(relationship);
       },
       _ => synerr!(
                span, "Unknown property for definition {}: {}",
                model::RelationAttributeDefinition::DEFINITION_NAME, name)
    }

    Ok(())
}


fn parse_number_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        return_type: model::ReturnType) -> Result<(), syn::Error>
{
    match def {
        model::AttributeDefinition::UnsignedSize(def) => parse_number_definition(def, name, content, span, return_type, false),
        model::AttributeDefinition::UnsignedInteger64(def) => parse_number_definition(def, name, content, span, return_type, false),
        model::AttributeDefinition::Integer64(def) => parse_number_definition(def, name, content, span, return_type, false),
        model::AttributeDefinition::Float64(def) => parse_number_definition(def, name, content, span, return_type, true),
        model::AttributeDefinition::UnsignedInteger32(def) => parse_number_definition(def, name, content, span, return_type, false),
        model::AttributeDefinition::Integer32(def) => parse_number_definition(def, name, content, span, return_type, true),
        model::AttributeDefinition::Float32(def) => parse_number_definition(def, name, content, span, return_type, false),
        _ => synerr!(span, "Mismatched definition for Num: {}", name)
    }
}

const DEFINITION_START: &'static str = "start";
const DEFINITION_INCREMENT: &'static str = "increment";
 
fn parse_number_definition<N>(
        def: &mut model::NumberAttributeDefinition<N>,
        name: &str,
        content: syn::parse::ParseBuffer,
        span: proc_macro2::Span,
        _return_type: model::ReturnType,
        is_float: bool) -> Result<(), syn::Error>
where
    N: FromStr,
    N::Err: std::fmt::Display
{
    macro_rules! parsenum {
        () => {
           if is_float {
                content.parse::<syn::LitFloat>()?.base10_parse()?
           } else {
                content.parse::<syn::LitInt>()?.base10_parse()?
           } 
        };
    }

    match name {
       DEFINITION_DEFAULT => {
            let n: N = parsenum!();
            def.default = Some(n)
       },
       DEFINITION_PRESET => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let preset = model::NumberPreset::from_str(&variant_name)
                .or(Err(mksynerr!(span, "Unknown definition preset for Num: {}", variant_name)))?;
            def.preset = Some(preset);
       },
       DEFINITION_START => {
            let n: N = parsenum!();
            def.start = Some(n)
       },
       DEFINITION_INCREMENT => {
            let n: N = parsenum!();
            def.increment = Some(n)
       },
       _ => synerr!(span, "Unknown definition property for Num: {}", name)
    }

    Ok(())
}

pub(crate) fn parse_variant(variant_name: &str, attr: &syn::Attribute, model: &model::EnumTrait)
        -> Result<model::VariantBuilder, syn::Error> {
    let mut variant_build = model::VariantBuilder::new();
    variant_build.name(variant_name.to_owned());
    attr.parse_nested_meta(|meta| {
        let attr_name = meta.path.get_ident()
            .ok_or(mksynerr!(attr.span(), "Invalid enum attribute: `{}`",
                meta.path.to_token_stream().to_string()))?
            .to_string();

        if variant_build.has_value(&attr_name) {
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
            model::AttributeDefinition::UnsignedInteger64(_) => model::Value::UnsignedInteger64(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::AttributeDefinition::Integer64(_) => model::Value::Integer64(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::AttributeDefinition::Float64(_) => model::Value::Float64(
                content.parse::<syn::LitFloat>()?.base10_parse()?),
            model::AttributeDefinition::UnsignedInteger32(_) => model::Value::UnsignedInteger32(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::AttributeDefinition::Integer32(_) => model::Value::Integer32(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::AttributeDefinition::Float32(_) => model::Value::Float32(
                content.parse::<syn::LitFloat>()?.base10_parse()?),
            model::AttributeDefinition::Byte(_) => model::Value::Byte(
                content.parse::<syn::LitByte>()?.value()),
            model::AttributeDefinition::FieldlessEnum(_) => model::Value::EnumVariant(
                content.parse::<model::Identifier>()?),
            model::AttributeDefinition::Relation(_) => model::Value::Relation(
                content.parse::<model::Identifier>()?),
            model::AttributeDefinition::Type(_) => model::Value::Type(
                content.parse::<model::Identifier>()?),
        };

        let attribute_value = model::AttributeValue::new(value);
        variant_build.value(attr_name, attribute_value);

        Ok(())
    })?;

    Ok(variant_build)
}

