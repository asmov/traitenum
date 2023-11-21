use traitenum::{self, enumtrait};

#[traitenum::enumtrait(crate::SimpleTrait)]
pub trait SimpleTrait {
    #[enumtrait::Str(default("spunko"))]
    fn name(&self) -> &'static str;
    fn column(&self) -> usize;
}

#[enumtrait(crate::ParentTrait)]
pub trait ParentTrait {
    type ChildsType: ChildTrait;
    type ChildsIteratorType: Iterator<Item = ChildsType>;

    #[enumtrait::Str(preset(Variant))]
    fn name(&self) -> &'static str;

    #[enumtrait::Rel(nature(OneToMany))]
    fn children(&self) -> Box<dyn Iterator<dyn ChildTrait<ParentTraitEnum = Self>>>;

    #[enumtrait::Rel(nature(OneToMany), dispatch(Static))]
    fn childs(&self) -> Self::ChildsIteratorType;
}

enum ImporterChildEnum {}
enum ChildsIteratorItem {
    ImporterFirstChildEnum(ImporterFirstChildEnum),
    ImporterSecondChildEnum(ImporterSecondChildEnum),
}

struct ChildsIterator {}
impl Iterator for ChildsIterator {
    type Item = ImporterChildEnum;

    fn next(&mut self) -> Option<Self::Item> {
        // actually in end-user code:
        /*
            ImporterParentEnum::Alpha.childs().map(|child_item| {
                let child_enum = match child_item { ChildsIteratorItem::ImporterChildEnum(enm) => enm, _ => unreachable!() };
                let child_enum = let_only!(child_item,  ChildsIteratorItem::ImporterChildEnum);
                child_enum.second_name();
            });
         */
    }
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
        let bytes = super::TRAITENUM_MODEL_BYTES_SIMPLETRAIT;
        let _model: traitenum_lib::model::EnumTrait = bincode::deserialize(bytes).unwrap();
    }
}
