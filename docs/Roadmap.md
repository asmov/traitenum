Roadmap
=======

Version 1
---------

### todo!()

- Cargo addon
  + cargo traitenum trait remove
  + cargo traitenum verify 
  + cargo traitenum fix
  + integration tests
- Derive macro gen shouldn't require a function name input
- Common Setting: optional(bool) => allows for Option<...> methods.
  + relationships: the one-side of relationships. the many-side can returns an empty iterator already
  + value on enum: e.g., #[traitenum(name(Some("foo")))] and #[traitenum(name(None))]
- Relationships
  + OnetoOne
- Implement a common TraitEnum trait for each enum, including:
  + type StaticIterator: Iterator<Item = Self>
  + variant_ordinal(&self) -> usize :: Num(preset(Ordinal))
  + variant_iter() -> Self::StaticIterator
  + variant_name(&self) -> &'static str :: Str(preset(Variant))
  + from_variant_name(&str) -> Option<Self>
- Polishing: Document, Refactor, Test
  + lib, macro, cargo
  + Make errors more helpful to end-users. Include tips 