use quote;
use syn;

mod model;

#[proc_macro_attribute]
pub fn enumtrait(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    /*let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let output = quote::quote!{};
    proc_macro::TokenStream::from(output)*/
    item
}

