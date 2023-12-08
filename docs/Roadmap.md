Roadmap
=======

Version 1
---------

### todo!()

- Support crate::paths aside from the root crate path
  + lib / macro
  + cargo
- Remove static dispatch code
  - Remove associated types
  - Restrict use of associated types
- One-to-One relationships
- Implement a common TraitEnum trait for each enum, including:
  + type StaticIterator: Iterator<Item = Self>
  + variant_ordinal(&self) -> usize :: Num(preset(Ordinal))
  + variant_iter() -> Self::StaticIterator
  + variant_name(&self) -> &'static str :: Str(preset(Variant))
  + from_variant_name(&str) -> Option<Self>
- Document, Refactor, Test
  + lib / macro
  + cargo
- Make proc-macro errors more helpful to end-users, including tips
