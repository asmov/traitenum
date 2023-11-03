use traitenum;
use traitenum_testcrate_exporter as exporter;

#[derive(traitenum::TraitEnum)]
#[traitenum::enumtrait(exporter::NameEnumTrait)]
enum NameEnum {
    #[traitenum::variant(name("The One"))]
    One,
    Two,
    #[traitenum::variant(name("Three"))]
    Three
}

#[cfg(test)]
mod tests {
    #[test]
    fn variant_methods() {
        assert_eq!("The One", NameEnum::One::name());
        assert_eq!("Two", NameEnum::Two::name());
        assert_eq!("Three", NameEnum::Three::name());
    }
}
