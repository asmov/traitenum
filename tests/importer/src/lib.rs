use traitenum_test_exporter_macro as traitmacro;
use traitenum_test_exporter_traits::SimpleTrait;
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

#[derive(traitmacro::ChildTraitEnum)]
#[traitenum(parent(ImporterParentEnum))]
pub enum ImporterChildEnum {
    One,
    Two,
    Three
}

#[cfg(test)]
mod tests {
    use traitenum_test_exporter_traits::SimpleTrait;

    #[test]
    fn test_enum_attributes() {
        assert_eq!("alpha", super::ImporterEnum::Alpha.name());
    }
}
