Roadmap
=======

Version 1
---------

### todo!()

- Support crate::paths aside from the root crate path
  + lib / macro
  + cargo
- Remove old static dispatch code
- Implement a common TraitEnum trait for each enum, including:
  + type StaticIterator: Iterator<Item = Self>
  + variant_iter() -> Self::StaticIterator
  + from_variant_name(&str) -> Option<Self>
  + variant_name(&self) -> &'static str :: Str(preset(Variant))
  + variant_ordinal(&self) -> usize :: Num(preset(Ordinal))
- Document, Refactor, Test
  + lib / macro
  + cargo
- Make proc-macro errors more helpful to end-users, including tips
