use traitenum;
use traitenum_testcrate_exporter_heavy as exporter;

#[derive(exporter::NameEnum)]
#[traitenum(trait(exporter::NameEnumTrait), model("name_enum"))]
enum MyNameEnum {
    #[variant(name("The One"))]
    One,
    Two,
    #[variant(name("Three"))]
    Three
}

#[cfg(test)]
mod tests {
    #[test]
    fn variant_methods() {
        assert_eq!("The One", MyNameEnum::One::name());
        assert_eq!("Two", MyNameEnum::Two::name());
        assert_eq!("Three", MyNameEnum::Three::name());
    }
}
