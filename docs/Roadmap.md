Roadmap
=======

Version 1
---------

### todo!()

- Derive macro gen shouldn't require a function name input
- Cargo adddon: Don't hardcode example trait on workspace new/init, use lib
- Cargo addon integration tests
- One-to-One relationships
- Implement a common TraitEnum trait for each enum, including:
  + type StaticIterator: Iterator<Item = Self>
  + variant_ordinal(&self) -> usize :: Num(preset(Ordinal))
  + variant_iter() -> Self::StaticIterator
  + variant_name(&self) -> &'static str :: Str(preset(Variant))
  + from_variant_name(&str) -> Option<Self>
- Document, Refactor, Test
  - Make errors more helpful to end-users. Include tips 
  + lib / macro
  + cargo
- Make proc-macro errors more helpful to end-users, including tips
