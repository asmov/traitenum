use proc_macro;
use proc_macro2;
use traitenum_test_exporter_macrolib as macrolib;


#[proc_macro_derive(ExporterTraitEnum, attributes(traitenum))]
pub fn derive_exporter_traitenum(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match macrolib::parse_derive_exporter_traitenum(proc_macro2::TokenStream::from(item)) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}