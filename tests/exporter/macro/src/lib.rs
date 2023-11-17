traitenum_lib::gen_require!(traitenum_test_exporter_traits, traitenum_test_exporter_macrolib);

traitenum_lib::gen_derive_macro!(SimpleTraitEnum, derive_traitenum_simple, traitlib::TRAITENUM_MODEL_BYTES_SIMPLETRAIT);
traitenum_lib::gen_derive_macro!(ChildTraitEnum, derive_traitenum_child, traitlib::TRAITENUM_MODEL_BYTES_CHILDTRAIT);
traitenum_lib::gen_derive_macro!(ParentTraitEnum, derive_traitenum_parent, traitlib::TRAITENUM_MODEL_BYTES_PARENTTRAIT);