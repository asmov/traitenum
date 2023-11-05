use traitenum;

mod models;

traitenum::gen::export_derive_macro!("name_enum", "NameEnum");
traitenum::gen::export_derive_macro!("alpha::bravo::column_enum", "ColumnEnum");
traitenum::gen::export_derive_macro!("alpha::name_enum", "NameEnum");
