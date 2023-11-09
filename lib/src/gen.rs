#[macro_export]
macro_rules! gen_require {
    ($traitlib_path:path, $macrolib_path:path) => {
        use proc_macro;
        use proc_macro2;
        use $traitlib_path as traitlib;
        use $macrolib_path as macrolib;
    };
}

#[macro_export]
macro_rules! gen_derive_macro {
    ($derive_name:ident, $model_bytes_path:path) => {
        #[proc_macro_derive($derive_name, attributes(traitenum))]
        pub fn derive_exporter_traitenum(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
            match macrolib::parse_derive_traitenum(proc_macro2::TokenStream::from(item), $model_bytes_path) {
                Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
                Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
            }
        }        
    };
}