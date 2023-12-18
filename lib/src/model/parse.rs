use std::str::FromStr;
use quote::{self, TokenStreamExt, ToTokens};
use syn::{self, parse};
use anyhow::{self, bail};

use crate::{model, error::Errors, span};

impl parse::Parse for model::Identifier {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let path: syn::Path = input.parse().map_err(|e| {
            syn::Error::new(span!(), format!("Unable to parse path. {}", e.to_string()))
        })?;

        Self::try_from(&path).map_err(|e| {
            syn::Error::new(span!(), format!(
                "Unable to parse an identifier from path `{}`. {}",
                path.to_token_stream().to_string(),
                e.to_string()))
        })
    }
}

impl From<&syn::Ident> for model::Identifier{
    fn from(ident: &syn::Ident) -> Self {
        model::Identifier::new(Vec::new(), ident.to_string())
    }
}

impl TryFrom<&syn::Path> for model::Identifier {
    type Error = Errors; 

    fn try_from(path: &syn::Path) -> Result<Self, Self::Error> {
        Self::try_from(path.clone())
    }
}

impl TryFrom<syn::Path> for model::Identifier{
    type Error = Errors;

    fn try_from(mut path: syn::Path) -> Result<Self, Self::Error> {
        let name = path.segments.pop().unwrap()
            .value().ident.to_string();
        let path = path.segments.pairs()
            .map(|pair| {
                if !pair.value().arguments.is_empty() {
                    Err(Errors::PathParsing(
                        "Path arguments are not supported".to_owned(), path.to_token_stream().to_string()).into())
                } else {
                    Ok(pair.value().ident.to_string())
                }
            })
            .collect::<Result<Vec<String>, Self::Error>>()?;

        Ok(Self::new(path, name))
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
        method_name: &str,
        return_type: model::ReturnType,
        return_type_id: Option<model::Identifier>
) -> anyhow::Result<model::AttributeDefinition> {
    // should be in the format of enumtrait::DefinitionName
    if attr.path().segments.len() != 2 {
        bail!(Errors::DefinitionParsing(
            method_name.to_string(),
            "Usage is `#[enumtrait::DefinitionType(...)]`. E.g: `#[enumtrait::Str()]`".to_string(),
            attr.path().to_token_stream().to_string()));
    }

    let definition_name = attr.path().segments.last().unwrap() // safe
        .ident.to_string();

    let mut def = model::AttributeDefinition::partial(Some(&definition_name), return_type, return_type_id)
        .map_err(|msg| Errors::DefinitionParsing(method_name.to_string(), msg, attr.to_token_stream().to_string()))?;

    // the parse function uses syn:Error, so our own errors have to be saved rather than unwrapped
    let mut parse_meta_err = None;
    let result = attr.parse_nested_meta(|meta| {
        let name = if let Some(ident) = meta.path.get_ident() { ident.to_string() } else {
            parse_meta_err = Some(Errors::DefinitionParsing(
                method_name.to_string(),
                format!("Unknown definition property: {}", meta.path.to_token_stream().to_string()),
                attr.to_token_stream().to_string()));
            return Err(meta.error("error"));
        };

        let content;
        syn::parenthesized!(content in meta.input);

        let result = match definition_name.as_str() {
            model::BoolAttributeDefinition::DEFINITION_NAME =>  
                parse_bool_attribute_definition(&mut def, &name, content, return_type),
            model::StaticStrAttributeDefinition::DEFINITION_NAME => 
                parse_string_attribute_definition(&mut def, &name, content, return_type),
            model::NumberAttributeDefinition::<usize>::DEFINITION_NAME => 
                parse_number_attribute_definition(&mut def, &name, content, return_type),
            model::FieldlessEnumAttributeDefinition::DEFINITION_NAME =>
                parse_enum_attribute_definition(&mut def, &name, content, return_type),
            model::RelationAttributeDefinition::DEFINITION_NAME =>
                parse_relation_attribute_definition(&mut def, &name, content, return_type),
             _ => {
                Err(Errors::UnknownDefinitionType(
                    definition_name.to_owned(),
                    method_name.to_owned(),
                    attr.to_token_stream().to_string()).into())
            }
        };

        Ok(())
    });

    if result.is_err() {
        // Expand on some of the errors to include the method name and token
        match parse_meta_err {
            Some(Errors::UnknownDefinitionSettingName{def_type, name}) =>
                bail!(Errors::UnknownDefinitionSetting{
                    method: method_name.to_owned(), def_type, setting: name, tokens: attr.to_token_stream().to_string()}),
            Some(Errors::InvalidDefinitionSettingValue{def_type, setting, value }) =>
                bail!(Errors::UnknownDefinitionSetting{
                    method: method_name.to_owned(), def_type, setting, tokens: attr.to_token_stream().to_string()}),
            Some(e) => bail!(e),
            None => bail!(result.unwrap_err())
        }
    }

    Ok(def)
}

const DEFINITION_DEFAULT: &'static str = "default";
const DEFINITION_PRESET: &'static str = "preset";

fn parse_bool_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType
    ) -> anyhow::Result<()> {
    let booldef = match def {
        model::AttributeDefinition::Bool(def) => def,
        _ => unreachable!("Mismatched definition for Bool: {}", name)
    };

    match name {
       DEFINITION_DEFAULT => {
            booldef.default = Some(content.parse::<syn::LitBool>()?.value())
       },
       _ => bail!(Errors::UnknownDefinitionSettingName{def_type: def.type_name().to_owned(), name: name.to_owned()})
    }

    Ok(())
}

fn parse_enum_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType
    ) -> anyhow::Result<()> {
    let enumdef = match def {
        model::AttributeDefinition::FieldlessEnum(def) => def,
        _ => unreachable!("Mismatched definition for Enum: {}", name)
    };

    match name {
       DEFINITION_DEFAULT => {
            let id: model::Identifier = content.parse()?;
            enumdef.default = Some(id)
       },
       _ => bail!(Errors::UnknownDefinitionSettingName{def_type: def.type_name().to_owned(), name: name.to_owned()})
    }

    Ok(())
}

fn parse_string_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType) -> anyhow::Result<()> {
    let strdef = match def {
        model::AttributeDefinition::StaticStr(def) => def,
        _ => unreachable!("Mismatched definition for Str: {}", name)
    };

    match name {
       DEFINITION_DEFAULT => {
            strdef.default = Some(content.parse::<syn::LitStr>()?.value())
       },
       DEFINITION_PRESET => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let preset = model::StringPreset::from_str(&variant_name)
                .or_else(|_| Err(Errors::UnknownDefinitionPresetName {def_type: def.type_name().to_owned(), name: variant_name}))?;
            strdef.preset = Some(preset);
       },
       _ => bail!(Errors::UnknownDefinitionSettingName{def_type: def.type_name().to_owned(), name: name.to_owned()})
    }

    Ok(())
}

const DEFINITION_NATURE: &'static str = "nature";
const DEFINITION_DISPATCH: &'static str = "dispatch";

fn parse_relation_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType) -> anyhow::Result<()>
{
    let reldef = def.get_relation_definition_mut();

    match name {
        DEFINITION_NATURE => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let nature = model::RelationNature::from_str(&variant_name)
                .or_else(|_| bail!(Errors::InvalidDefinitionSettingValue{
                    def_type: def.type_name().to_owned(),
                    setting: DEFINITION_NATURE.to_owned(),
                    value: variant_name}))?;
            reldef.nature = Some(nature);
        },
        DEFINITION_DISPATCH => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let dispatch = model::Dispatch::from_str(&variant_name)
                .or_else(|_| bail!(Errors::InvalidDefinitionSettingValue{
                    def_type: def.type_name().to_owned(),
                    setting: DEFINITION_DISPATCH.to_owned(),
                    value: variant_name}))?;
            reldef.dispatch = Some(dispatch);
        },
       _ => bail!(Errors::UnknownDefinitionSettingName{def_type: def.type_name().to_owned(), name: name.to_owned()})
    }

    Ok(())
}

fn parse_number_attribute_definition(
        def: &mut model::AttributeDefinition,
        name: &str,
        content: syn::parse::ParseBuffer,
        return_type: model::ReturnType) -> anyhow::Result<()>
{
    match def {
        model::AttributeDefinition::UnsignedSize(numdef) => parse_number_definition(def, numdef, name, content, return_type, false),
        model::AttributeDefinition::UnsignedInteger64(numdef) => parse_number_definition(def, numdef, name, content, return_type, false),
        model::AttributeDefinition::Integer64(numdef) => parse_number_definition(def, numdef, name, content, return_type, false),
        model::AttributeDefinition::Float64(numdef) => parse_number_definition(def, numdef, name, content, return_type, true),
        model::AttributeDefinition::UnsignedInteger32(numdef) => parse_number_definition(def, numdef, name, content, return_type, false),
        model::AttributeDefinition::Integer32(numdef) => parse_number_definition(def, numdef, name, content, return_type, true),
        model::AttributeDefinition::Float32(numdef) => parse_number_definition(def, numdef, name, content, return_type, false),
        _ => unreachable!("Mismatched definition for Num: {}", name)
    }
}

const DEFINITION_START: &'static str = "start";
const DEFINITION_INCREMENT: &'static str = "increment";
 
fn parse_number_definition<N>(
        def: &mut model::AttributeDefinition,
        numdef: &mut model::NumberAttributeDefinition<N>,
        name: &str,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType,
        is_float: bool) -> anyhow::Result<()>
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
            numdef.default = Some(n)
       },
       DEFINITION_PRESET => {
            let variant_name = content.parse::<syn::Ident>()?.to_string();
            let preset = model::NumberPreset::from_str(&variant_name)
                .or_else(|_| bail!(Errors::UnknownDefinitionPresetName {def_type: def.type_name().to_owned(), name: variant_name}))?;
            numdef.preset = Some(preset);
       },
       DEFINITION_START => {
            let n: N = parsenum!();
            numdef.start = Some(n)
       },
       DEFINITION_INCREMENT => {
            let n: N = parsenum!();
            numdef.increment = Some(n)
       },
       _ => bail!(Errors::UnknownDefinitionSettingName{def_type: def.type_name().to_owned(), name: name.to_owned()})
    }

    Ok(())
}

pub(crate) fn parse_variant(
    variant_name: &str,
    attr: &syn::Attribute,
    model: &model::EnumTrait
) -> anyhow::Result<model::VariantBuilder> {
    let mut variant_build = model::VariantBuilder::new();
    variant_build.name(variant_name.to_owned());

    let mut meta_error: Option<Errors> = None;
    attr.parse_nested_meta(|meta| {
        let attr_name = meta.path.get_ident()
            .ok_or_else(|| {
                meta_error = Some(Errors::UnexpectedParsing{
                    expected: "Variant setting name".to_owned(),
                    found: meta.path.to_token_stream().to_string(),
                    tokens: attr.to_token_stream().to_string()});
                meta.error("error")
            })?
            .to_string();

        if variant_build.has_value(&attr_name) {
            meta_error = Some(Errors::DuplicateParsing(
                "Variant setting".to_owned(),
                attr_name,
                attr.to_token_stream().to_string()));
            return Err(meta.error("error"));
        }

        let method = model.methods().iter().find(|m| m.name() == attr_name)
            .ok_or(mksynerr!("Unknown enum attribute: {}", attr_name))?;

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
            model::AttributeDefinition::FieldlessEnum(enumdef) => {
                let mut id = content.parse::<model::Identifier>()?;
                // users are allowed to drop the enum type in short-hand (Foo instead of MyEnum::Foo)
                // fill in the path if they do this
                if id.path().is_empty() {
                    id = enumdef.identifier.append(id)
                }

                model::Value::EnumVariant(id)
            },
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

impl quote::ToTokens for model::AttributeValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            match &self.value() {
                model::Value::Bool(b) => quote::quote!(#b),
                model::Value::StaticStr(s) => quote::quote!(#s),
                model::Value::UnsignedSize(n) => quote::quote!(#n),
                model::Value::UnsignedInteger64(n) => quote::quote!(#n),
                model::Value::Integer64(n) => quote::quote!(#n),
                model::Value::Float64(n) => quote::quote!(#n),
                model::Value::UnsignedInteger32(n) => quote::quote!(#n),
                model::Value::Integer32(n) => quote::quote!(#n),
                model::Value::Float32(n) => quote::quote!(#n),
                model::Value::Byte(n) => quote::quote!(#n),
                model::Value::EnumVariant(id) => id.to_token_stream(),
                model::Value::Relation(id) => id.to_token_stream(),
                model::Value::Type(id) => id.to_token_stream(),
            }
        );
    }
}

impl From<model::Identifier> for syn::Path {
    fn from(value: model::Identifier) -> Self {
        Self::from(&value)
    }
}

impl From<&model::Identifier> for syn::Path {
    fn from(value: &model::Identifier) -> Self {
        let mut path = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::new()
        };

        value.path.iter().for_each(|s| {
                let ident = syn::Ident::new(s, span!());
                let segment = syn::PathSegment::from(ident);
                path.segments.push(segment)
            }
        );

        let ident = syn::Ident::new(value.name(), span!());
        let segment = syn::PathSegment::from(ident);
        path.segments.push(segment);
 
        path

    }
}

/// Using this with the following return types will panic!():
///   - ReturnType::BoxedTrait
///   - ReturnType::BoxedTraitIterator
///   - ReturnType::AssociatedType
///   - ReturnType::Type
impl quote::ToTokens for model::ReturnType{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            match &self {
                model::ReturnType::Bool => quote::quote!{ bool },
                model::ReturnType::StaticStr => quote::quote!{ &'static str },
                model::ReturnType::UnsignedSize => quote::quote!{ usize },
                model::ReturnType::UnsignedInteger64 => quote::quote!{ u64 },
                model::ReturnType::Integer64 => quote::quote!{ i64 },
                model::ReturnType::Float64 => quote::quote!{ f64 },
                model::ReturnType::UnsignedInteger32 => quote::quote!{ u32 },
                model::ReturnType::Integer32 => quote::quote!{ i32 },
                model::ReturnType::Float32 => quote::quote!{ f32 },
                model::ReturnType::Byte => quote::quote!{ u8 },
                // this has to be handled conditionally
                model::ReturnType::BoxedTrait => unreachable!("ReturnType::BoxedTrait cannot directly produce a TokenStream"),
                model::ReturnType::BoxedTraitIterator => unreachable!("ReturnType::BoxedTraitIterator cannot directly produce a TokenStream"),
                model::ReturnType::AssociatedType => unreachable!("ReturnType::AssociatedType cannot directly produce a TokenStream"),
                model::ReturnType::Enum => unreachable!("ReturnType::Enum cannot directly produce a TokenStream"),
                model::ReturnType::Type => unreachable!("ReturnType::Type cannot directly produce a TokenStream")
            }
        );
    }
}

impl quote::ToTokens for model::Identifier {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(syn::Path::from(self).to_token_stream())
    }
}

impl model::Method {
    pub fn return_type_tokens(&self) -> proc_macro2::TokenStream {
        match self.return_type {
            model::ReturnType::BoxedTrait => {
                let ident = self.attribute_definition()
                    .get_relation_definition()
                    .identifier()
                    .to_token_stream();

                quote::quote!{
                    ::std::boxed::Box<dyn #ident>
                }
            },
            model::ReturnType::BoxedTraitIterator => {
                let ident = self.attribute_definition()
                    .get_relation_definition()
                    .identifier()
                    .to_token_stream();

                quote::quote!{
                    ::std::boxed::Box<dyn ::std::iter::Iterator<Item = ::std::boxed::Box<dyn #ident>>>
                }
            },
            model::ReturnType::Type => {
                match self.attribute_definition() {
                    model::AttributeDefinition::FieldlessEnum(enumdef) => enumdef.identifier.to_token_stream(),
                    // statically dispatched relations
                    model::AttributeDefinition::Relation(reldef) => reldef.identifier.to_token_stream(),
                    _ => unreachable!("Invalid attribute definition for ReturnType::Type")
                }
            },
            _ => self.return_type.to_token_stream()
        }
    }
}