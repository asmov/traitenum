use quote::{self, TokenStreamExt, ToTokens};

use crate::model;

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

impl model::Identifier {
    pub fn to_path(&self, spanned: &impl syn::spanned::Spanned) -> syn::Path {
        let mut path = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::new()
        };

        self.path.iter().for_each(|s| {
                let ident = syn::Ident::new(s, spanned.span());
                let segment = syn::PathSegment::from(ident);
                path.segments.push_value(segment)
            }
        );

        let ident = syn::Ident::new(self.name(), spanned.span());
        let segment = syn::PathSegment::from(ident);
        path.segments.push(segment);
 
        path
    }
}

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
                model::ReturnType::Type => unreachable!("ReturnType::Type cannot directly produce a TokenStream")
            }
        );
    }
}

impl quote::ToTokens for model::Identifier {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
       
        tokens.append_all(self.to_path(tokens).to_token_stream())
    }
}

impl model::Method {
    pub fn return_type_tokens(&self) -> proc_macro2::TokenStream {
        match self.return_type {
            model::ReturnType::Type => {
                match self.attribute_definition() {
                    model::AttributeDefinition::FieldlessEnum(enumdef) => enumdef.identifier.to_token_stream(),
                    model::AttributeDefinition::Relation(reldef) => reldef.identifier.to_token_stream(),
                    _ => unreachable!("Invalid attribute definition for ReturnType::Type")
                }
            },
            _ => self.return_type().to_token_stream()
        }
    }
}