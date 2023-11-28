use traitenum_test_exporter_macro as traitmacro;
use traitenum_test_exporter_traits::SimpleTrait;
use traitenum_test_exporter_traits::ParentTrait;
use traitenum_test_exporter_traits::ChildTrait;

#[derive(traitmacro::SimpleTraitEnum)]
pub enum ImporterEnum {
    #[traitenum(name("alpha"), column(0))]
    Alpha,
    #[traitenum(column(2))]
    Bravo,
    #[traitenum(name("charles"), column(4))]
    Charlie
}

#[derive(traitmacro::ParentTraitEnum)]
pub enum ImporterParentEnum {
    #[traitenum(children(ImporterChildAlphaEnum))]
    Alpha,
    #[traitenum(children(ImporterChildAlphaEnum))]
    Bravo,
    #[traitenum(children(ImporterChildAlphaEnum))]
    Charlie 
}


#[derive(traitmacro::ChildTraitEnum)]
#[traitenum(parent(ImporterParentEnum::Bravo))]
pub enum ImporterChildAlphaEnum {
    Zero,
    One,
    Two,
}

#[cfg(test)]
mod tests {
    use traitenum_test_exporter_traits::{SimpleTrait,ChildTrait,ParentTrait};

    #[test]
    fn test_enum_attributes() {
        assert_eq!("alpha", super::ImporterEnum::Alpha.name());
        assert_eq!("One", super::ImporterChildAlphaEnum::One.topic());
        assert_eq!("Charlie", super::ImporterParentEnum::Charlie.name());
    }

    #[test]
    fn test_enum_iterators() {
        for child_variant in super::ImporterParentEnum::Alpha.children() {
            match child_variant.ordinal() {
                0 => assert_eq!("Zero", child_variant.topic()),
                1 => assert_eq!("One", child_variant.topic()),
                2 => assert_eq!("Two", child_variant.topic()),
                _ => panic!("Unknown ordinal")
            }
        }

        // test order
        assert_eq!("One", super::ImporterParentEnum::Alpha.children().collect::<Vec<_>>()[1].topic());
    }

    #[test]
    fn test_enum_many_to_one() {
        assert_eq!("Bravo", super::ImporterChildAlphaEnum::Two.parent().name());
    }

    #[test]
    fn test_default_impl_fn() {
        assert_eq!("charles :: 4", super::ImporterEnum::Charlie.default_impl());
    }
}
