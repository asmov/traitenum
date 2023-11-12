use quote::{self, TokenStreamExt};

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
                model::Value::Type(id) => todo!("tokenize identifier"),
                model::Value::Relation(id) => todo!("tokenize identifier"),
            }
        );
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
                model::ReturnType::Type => todo!("return sig: type"),
                model::ReturnType::TypeReference => todo!("return sig type reference"),
            }
        );
    }
}

