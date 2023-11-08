use traitenum;

#[traitenum]
trait NameEnumTrait {
    #[traitenum(variant("Name"), type(str), default(variant))]
    fn name(&self) -> &'static str;
}
