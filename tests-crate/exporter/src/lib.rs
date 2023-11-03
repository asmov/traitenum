use traitenum;


traitenum::enumtrait!{
    column: str { default: variant },
}

trait NameEnumTrait {
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use traitenum;

    #[test]
    fn trait_concrete() {
        #[derive(traitenum::TraitEnum)]
        #[traitenum::enumtrait(NameEnumTrait)]
        enum NameEnum {
            #[traitenum::variant(name("The One"))]
            One,
            Two,
            #[traitenum::variant(name("Three"))]
            Three
        }

        assert_eq!("The One", NameEnum::One::name());
        assert_eq!("Two", NameEnum::Two::name());
        assert_eq!("Three", NameEnum::Three::name());
    }
}
