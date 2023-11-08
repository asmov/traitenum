use quote::{self, ToTokens};
use syn::spanned::Spanned;
use syn::{self, parse};
use proc_macro2;

mod model;

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

#[proc_macro_attribute]
pub fn enumtrait(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse_enumtrait(attr, item) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}

fn parse_enumtrait(attr: proc_macro::TokenStream, item: proc_macro::TokenStream)
        -> Result<proc_macro2::TokenStream, syn::Error> {
    let identifier: model::Identifier = syn::parse(attr)?;
    
    let mut item: syn::ItemTrait = syn::parse(item)?;
    if identifier.name() != item.ident.to_string() {
        return Err(syn::Error::new(item.ident.span(),
                format!("Trait name does not match #enumtrait(<absolute trait path>): {}", identifier.name())));
    }

    //dbg!(&item.ident);
    //item.items.iter().for_each(|a| { dbg!(proc_macro::TokenStream::from(a.to_token_stream())); } );

    //let mut methods: Vec<model::Method> = Vec::new(); 

    for trait_item in &mut item.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                // ignore functions with default implementations
                if func.default.is_some() {
                    continue;
                }

                for a in &func.attrs {
                    dbg!(proc_macro::TokenStream::from(a.to_token_stream()));
                }

                func.attrs.clear();

                match &func.sig.output {
                    syn::ReturnType::Default => return Err(syn::Error::new(trait_item.span(),
                        "enumtrait: Default return types are not supported")),
                    syn::ReturnType::Type(_, _return_type) => {/* todo */},
                }

                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            syn::TraitItem::Macro(_) => {},
            syn::TraitItem::Verbatim(_) => {},
            syn::TraitItem::Const(_) => {},
            syn::TraitItem::Type(_) => {
                dbg!("Type");
                //dbg!(proc_macro::TokenStream::from(it.to_token_stream()));
            },
            _ => todo!()
        }
    }

   Ok(item.to_token_stream())
}

impl parse::Parse for model::EnumTrait {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let return_type: syn::Type = input.parse()?;
        dbg!(return_type.to_token_stream());
        let content;
        let _brace = syn::braced!(content in input);
        let _fields = content.parse_terminated(Field::parse, syn::Token![,]);
        Ok(Self {
            identifer: model::Identifier::new(Vec::new(), name.to_string()),
            methods: Vec::new()
        })
    }
}

struct Field {

}

impl parse::Parse for Field {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        dbg!(key.to_token_stream());
        input.parse::<syn::Token![:]>()?;
        let value: syn::Expr = input.parse()?;
        dbg!(value.to_token_stream());
        Ok(Self {
        })
    }
}

