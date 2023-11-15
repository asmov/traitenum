use traitenum::{self, enumtrait};

#[traitenum::enumtrait(crate::SimpleTrait)]
pub trait SimpleTrait {
    #[enumtrait::Str(default("spunko"))]
    fn name(&self) -> &'static str;
    fn column(&self) -> usize;
}

#[enumtrait(crate::ParentTrait)]
pub trait ParentTrait {
    #[enumtrait::Str(preset(Variant))]
    fn name(&self) -> &'static str;
}

#[enumtrait(crate::ChildTrait)]
pub trait ChildTrait {
    type ParentTraitEnum: ParentTrait; 

    #[enumtrait::Num(preset(Ordinal))]
    fn ordinal(&self) -> usize;

    #[enumtrait::Rel(relationship(ManyToOne))]
    fn parent(&self) -> Self::ParentTraitEnum;
}


#[cfg(test)]
mod tests {
    use traitenum_lib;
    use bincode;

    #[test]
    fn test_load_model() {
        let bytes = super::TRAITENUM_SIMPLETRAIT;
        let _model: traitenum_lib::model::EnumTrait = bincode::deserialize(bytes).unwrap();
    }
}
