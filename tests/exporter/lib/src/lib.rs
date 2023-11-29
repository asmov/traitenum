use traitenum::{self, enumtrait};

#[traitenum::enumtrait(crate::SimpleTrait)]
pub trait SimpleTrait {
    #[enumtrait::Str(default("spunko"))]
    fn name(&self) -> &'static str;
    fn column(&self) -> usize;

    fn default_impl(&self) -> String {
        format!("{} :: {}", self.name(), self.column())
    }
}

#[enumtrait(crate::ParentTrait)]
pub trait ParentTrait {
    #[enumtrait::Str(preset(Variant))]
    fn name(&self) -> &'static str;

    #[enumtrait::Rel(nature(OneToMany))]
    fn children(&self) -> Box<dyn Iterator<Item = Box<dyn ChildTrait>>>;
}

#[enumtrait(crate::ChildTrait)]
pub trait ChildTrait {
    #[enumtrait::Str(preset(Variant))]
    fn topic(&self) -> &'static str;

    #[enumtrait::Num(preset(Ordinal))]
    fn ordinal(&self) -> usize;

    #[enumtrait::Rel(nature(ManyToOne))]
    fn parent(&self) -> Box<dyn ParentTrait>;
}

#[cfg(test)]
mod tests {
    use traitenum_lib;
    use bincode;

    #[test]
    fn test_load_model() {
        let bytes = super::TRAITENUM_MODEL_BYTES_SIMPLETRAIT;
        let _model: traitenum_lib::model::EnumTrait = bincode::deserialize(bytes).unwrap();
    }
}
