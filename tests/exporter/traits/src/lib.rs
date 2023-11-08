use traitenum;

#[traitenum::enumtrait(crate::SimpleTrait)]
pub trait SimpleTrait {
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use traitenum_lib;
    use bincode;

    #[test]
    fn test_load_model() {
        let bytes = super::MODEL_SIMPLETRAIT;
        let _model: traitenum_lib::model::EnumTrait = bincode::deserialize(bytes).unwrap();
    }
}
