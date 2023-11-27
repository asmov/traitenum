const MODEL_BYTES_NAME: &'static str = "TRAITENUM_MODEL_BYTES_";

mod traitenum;
mod enumtrait;

pub use traitenum::traitenum_derive_macro;
pub use enumtrait::enumtrait_macro;

#[cfg(test)]
mod tests {
    use quote;
    use crate::{TRAIT_ATTRIBUTE_HELPER_NAME, model, macros::enumtrait, macros::traitenum};


    /// Asserts that the expected value has been defined for a given enum variant
    macro_rules! assert_traitenum_value {
        ($model:ident, $variant_name:literal, $attribute_name:literal, $value_type:ident, $expected:expr) => {
            assert!($model.variant($variant_name).is_some(), "Variant doesn't exist: {}", $variant_name);
            assert!($model.variant($variant_name).unwrap().value($attribute_name).is_some(),
                "Variant attribute doesn't exist: {} -> {}", $variant_name, $attribute_name);
            match $model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                model::Value::$value_type(ref val) => assert_eq!($expected, *val),
                _ => assert!(false, "Incorrect value type for attribute: {}", $attribute_name)
            }
        };
    }

    /// Asserts that the expected enum value has been defined for a given enum variant
    macro_rules! assert_traitenum_value_enum {
        ($model:ident, $variant_name:literal, $attribute_name:literal, $expected:literal) => {
            match $model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                model::Value::EnumVariant(ref val) => assert_eq!($expected, val.to_string()),
                _ => assert!(false, "Incorrect value type for attribute: {}", $attribute_name)
            }
        };
    }

    #[test]
    fn test_parse_enumtrait() {
        let attribute_src = quote::quote!{
            crate::tests::MyTrait
        };

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test Str default
                #[enumtrait::Str(default(":)"))]
                fn str_default(&self) -> &'static str;
                // test Num default
                #[enumtrait::Num(default(44))]
                fn num_default(&self) -> usize;
               // test Bool default
                #[enumtrait::Bool(default(true))]
                fn bool_default(&self) -> bool;
                // test Enum default
                #[enumtrait::Enum(default(RPS::Rock))]
                fn enum_default(&self) -> RPS;
                // test Str variant preset
                #[enumtrait::Str(preset(Variant))]
                fn str_preset_variant(&self) -> &'static str;
                // test Num serial preset w/start and increment 
                #[enumtrait::Num(preset(Serial), start(3), increment(2))]
                fn num_preset_serial_all(&self) -> u64;
                // test Rel dynamic many-to-one
                #[enumtrait::Rel(nature(ManyToOne), dispatch(Dynamic))]
                fn many_to_one_dyn(&self) -> Box<dyn FirstOneTrait>;
                 // test Rel dynamic one-to-many
                #[enumtrait::Rel(nature(OneToMany), dispatch(Dynamic))]
                fn one_to_many_dyn(&self) -> Box<dyn Iterator<Item = dyn FirstManyTrait>>;
                // test elided Rel dynamic one-to-many
                fn one_to_many_elided_dyn(&self) -> Box<dyn Iterator<Item = dyn SecondManyTrait>>;
                // test Rel many-to-one dynamic (elided)
                #[enumtrait::Rel(nature(ManyToOne))]
                fn many_to_one_dynelide(&self) -> Box<dyn SecondOneTrait>;
                // test default implementation
                fn default_implementation(&self) {
                    todo!();
                }
            }
        };
        
        let model = enumtrait::parse_enumtrait_macro(attribute_src, item_src).unwrap().model;
        dbg!(&model);

        assert_eq!(vec!["crate", "tests"], model.identifier().path());
        assert_eq!("MyTrait", model.identifier().name());

        let item_src = quote::quote!{
            #[traitenum(many_to_one(ManyToOneEnum::My))]
            enum MyEnum {
                #[traitenum(one_to_many(OneToManyOneEnum))]
                One,
                // test short-hand enum values
                #[traitenum(str_preset_variant("2"), enum_default(Paper), one_to_many(OneToManyTwoEnum))]
                Two,
                #[traitenum(bool_default(false), one_to_many(OneToManyThreeEnum))]
                Three,
                #[traitenum(enum_default(RPS::Scissors), one_to_many(OneToManyFourEnum))]
                Four,
            }
        };

        let model_bytes = bincode::serialize(&model).unwrap();
        let traitenum::TraitEnumMacroOutput {model: enum_model, tokens: enum_tokens} = traitenum::parse_traitenum_macro(
            item_src, &model_bytes).unwrap();

        dbg!(&enum_model);
        dbg!(&enum_tokens.to_string());

        // test defaults
        assert_traitenum_value!(enum_model, "One", "str_default", StaticStr, ":)");
        assert_traitenum_value!(enum_model, "One", "bool_default", Bool, true);
        assert_traitenum_value!(enum_model, "One", "num_default", UnsignedSize, 44);
        assert_traitenum_value_enum!(enum_model, "One", "enum_default", "RPS::Rock");
        // test string preset(variant)
        assert_traitenum_value!(enum_model, "Two", "str_preset_variant", StaticStr, "2");
        // test u64 preset(serial) w/start(3), increment(2)
        assert_traitenum_value!(enum_model, "Three", "num_preset_serial_all", UnsignedInteger64, 7);
        // test short-hand enum value
        assert_traitenum_value_enum!(enum_model, "Two", "enum_default", "RPS::Paper");
        // test non-default bool
        assert_traitenum_value!(enum_model, "Three", "bool_default", Bool, false);
        // test non-default enum
        assert_traitenum_value_enum!(enum_model, "Four", "enum_default", "RPS::Scissors");
    }

    #[test]
    fn test_parse_enumtrait_errors() {
        let simple_attribute_src = quote::quote!{
            crate::tests::MyTrait
        };
        let simple_item_src = quote::quote!{
            pub trait MyTrait {
                fn name(&self) -> &'static str;
            }
        };

        // test error: empty identifier
        let attribute_src = quote::quote!{};
        assert!(enumtrait::parse_enumtrait_macro(attribute_src, simple_item_src.clone()).is_err(),
            "Empty #[{}(<pathspec>)] should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);
        
        // test error: mismatched trait name with identifier
        let attribute_src = quote::quote!{ crate::tests::TheirTrait };
        assert!(enumtrait::parse_enumtrait_macro(attribute_src, simple_item_src.clone()).is_err(),
            "Mismatched trait name and #[{}(<pathspec>)] identifier should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);

        let unimplemented_static_dispatch_src = quote::quote!{
            pub trait MyTrait {
                type ManyType: ManyTrait;

                #[traitenum::Rel(dispatch(Static))]
                fn many_to_one(&self) -> Self::ManyType;
            }
        };

        assert!(enumtrait::parse_enumtrait_macro(
            simple_attribute_src.clone(),
            unimplemented_static_dispatch_src).is_err(),
            "Static dispatch is not currently supported and should throw an Error");

        let unimplemented_implied_static_dispatch_src = quote::quote!{
            pub trait MyTrait {
                type ManyType: ManyTrait;

                fn many_to_one(&self) -> Self::ManyType;
            }
        };

        assert!(enumtrait::parse_enumtrait_macro(
            simple_attribute_src.clone(),
            unimplemented_implied_static_dispatch_src).is_err(),
            "Implied static dispatch is not currently supported and should throw an Error");

        

    }

    #[test]
    fn test_traitenum_macro() {

    }
}
