use traitenum_test_exporter_macro as traitmacro;
use traitenum_test_exporter_traits::SimpleTrait;

#[derive(traitmacro::SimpleTraitEnum)]
pub enum ImporterEnum {
    #[traitenum(name("alpha"))]
    Alpha,
    Bravo,
    Charlie
}

#[cfg(test)]
mod tests {
    use traitenum_test_exporter_traits::SimpleTrait;

    #[test]
    fn test_enum_attributes() {
        assert_eq!("alpha", super::ImporterEnum::Alpha.name());
    }
}
