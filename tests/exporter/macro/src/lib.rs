use proc_macro;
use std::sync::OnceLock;
use traitenum_lib::model as model;

use traitenum_test_exporter_lib as lib;

#[proc_macro_derive(ExporterTraitEnum, attributes(traitenum))]
pub fn derive_exporter_traitenum(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    static MODEL: OnceLock<model::EnumTrait> = OnceLock::new();
    let model = MODEL.get_or_init(|| model::EnumTrait::from(lib::MODEL_SIMPLETRAIT as &'static [u8]));
    item
}