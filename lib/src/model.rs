use std::{fmt::Display, str::FromStr, collections::HashMap, collections::hash_map};

use serde;
use convert_case::{self as case, Casing};

pub mod parse;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnumTrait {
    identifier: Identifier,
    methods: Vec<Method>,
    types: Vec<AssociatedType>
}

impl EnumTrait {
    pub fn identifier(&self) -> &Identifier { &self.identifier }
    pub fn methods(&self) -> &[Method] { &self.methods }
    pub fn types(&self) -> &[AssociatedType] { &self.types }
    
    pub fn relation_methods(&self) -> Vec<(&Method, &RelationAttributeDefinition)> {
        self.methods.iter()
            .filter_map(|method|
                match method.attribute_definition {
                    AttributeDefinition::Relation(ref relation_def) => Some((method, relation_def)),
                    _ => None
                }
            )
            .collect()
    }

    pub const fn new(identifier: Identifier, methods: Vec<Method>, types: Vec<AssociatedType>) -> Self {
        Self {
            identifier,
            methods,
            types: types
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Identifier{
    path: Vec<String>,
    name: String
}

impl Identifier {
    pub fn path(&self) -> &[String] { &self.path }
    pub fn name(&self) -> &str { &self.name }

    pub const fn new(path: Vec<String>, name: String) -> Self {
        Self {
            path,
            name
        }
    }

    pub fn base(&self) -> Option<Identifier> {
        let mut path = self.path.clone();
        if let Some(name) = path.pop() {
            Some(Self::new(path, name))
        } else {
            None
        }
    }

    pub fn append(&self, rh: Identifier) -> Identifier {
        let mut path = self.path.clone();
        path.push(self.name.to_owned());
        path.extend_from_slice(&rh.path);
        Self::new(path, rh.name().to_owned())
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut path = self.path.clone();
        path.push(self.name.to_owned());
        write!(f, "{}", path.join("::"))
    }
}

pub(crate) struct AssociatedTypePartial {
    pub(crate) name: String,
    pub(crate) trait_identifier: Identifier,
    pub(crate) matched: bool
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssociatedType {
    name: String,
    relation_name: String,
    trait_identifier: Identifier,
    nature: RelationNature
}

impl AssociatedType {
    pub fn name(&self) -> &str { &self.name }
    pub fn relation_name(&self) -> &str { &self.relation_name }
    pub fn trait_identifier(&self) -> &Identifier { &self.trait_identifier }
    pub fn nature(&self) -> RelationNature { self.nature }

    pub fn valid_return_type_id(identifier: &Identifier) -> bool {
        identifier.path.len() == 1 && identifier.path[1] == "Self"
    }

    pub const fn new(
        name: String,
        relation_name: String,
        trait_identifier: Identifier,
        nature: RelationNature) -> Self
    {
        Self {
            name,
            relation_name: relation_name,
            trait_identifier,
            nature
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReturnType {
    Bool,
    StaticStr,
    UnsignedSize,
    UnsignedInteger64,
    Integer64,
    Float64,
    UnsignedInteger32,
    Integer32,
    Float32,
    Byte,
    BoxedTrait,
    BoxedTraitIterator,
    AssociatedType,
    Enum,
    Type
}

impl ReturnType {
    pub const BOOL: &'static str = "bool";
    pub const STATIC_STR: &'static str = "&'static str";
    pub const UNSIGNED_SIZE: &'static str = "usize";
    pub const UNSIGNED_INTEGER_64: &'static str = "u64";
    pub const INTEGER_64: &'static str = "i64";
    pub const FLOAT_64: &'static str = "f64";
    pub const UNSIGNED_INTEGER_32: &'static str = "u32";
    pub const INTEGER_32: &'static str = "i32";
    pub const FLOAT_32: &'static str = "f32";
    pub const BYTE: &'static str = "u8";
}

impl Display for ReturnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReturnType::Bool => f.write_str(Self::BOOL),
            ReturnType::StaticStr => f.write_str(Self::STATIC_STR),
            ReturnType::UnsignedSize => f.write_str(Self::UNSIGNED_SIZE),
            ReturnType::UnsignedInteger64 => f.write_str(Self::UNSIGNED_INTEGER_64),
            ReturnType::Integer64 => f.write_str(Self::INTEGER_64),
            ReturnType::Float64 => f.write_str(Self::FLOAT_64),
            ReturnType::UnsignedInteger32 => f.write_str(Self::UNSIGNED_INTEGER_32),
            ReturnType::Integer32 => f.write_str(Self::INTEGER_32),
            ReturnType::Float32 => f.write_str(Self::FLOAT_32),
            ReturnType::Byte => f.write_str(Self::BYTE),
            // complex types
            ReturnType::BoxedTrait => write!(f, "Box<dyn Trait>"),
            ReturnType::BoxedTraitIterator => write!(f, "Box<dyn Iterator<Item = dyn Trait>"),
            ReturnType::AssociatedType => write!(f, "<Self::Type>"),
            ReturnType::Enum => write!(f, "<Enum>"),
            ReturnType::Type => write!(f, "<Type>"),
        }
    }
}

impl FromStr for ReturnType {
    type Err = ();

    /// Matches on primitive supported types only.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::BOOL => Ok(Self::Bool),
            Self::STATIC_STR => Ok(Self::StaticStr),
            Self::UNSIGNED_SIZE => Ok(Self::UnsignedSize),
            Self::UNSIGNED_INTEGER_64 => Ok(Self::UnsignedInteger64),
            Self::INTEGER_64 => Ok(Self::Integer64),
            Self::FLOAT_64 => Ok(Self::Float64),
            Self::UNSIGNED_INTEGER_32 => Ok(Self::UnsignedInteger32),
            Self::INTEGER_32 => Ok(Self::Integer32),
            Self::FLOAT_32 => Ok(Self::Float32),
            Self::BYTE => Ok(Self::Byte),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AttributeDefinition {
    Bool(BoolAttributeDefinition),
    StaticStr(StaticStrAttributeDefinition),
    UnsignedSize(NumberAttributeDefinition<usize>),
    UnsignedInteger64(NumberAttributeDefinition<u64>),
    Integer64(NumberAttributeDefinition<i64>),
    Float64(NumberAttributeDefinition<f64>),
    UnsignedInteger32(NumberAttributeDefinition<u32>),
    Integer32(NumberAttributeDefinition<i32>),
    Float32(NumberAttributeDefinition<f32>),
    Byte(NumberAttributeDefinition<u8>),
    FieldlessEnum(FieldlessEnumAttributeDefinition),
    Relation(RelationAttributeDefinition),
    Type(TypeAttributeDefinition)
}

impl AttributeDefinition {
    pub fn partial(definition_name: Option<&str>, return_type: ReturnType, return_identifier: Option<Identifier>)
            -> Result<Self, String> {

        macro_rules! chk_defname {
            ($expected:path) => {
               if let Some(defname) = definition_name {
                    if (defname != $expected) {
                        return Err(format!("Definition type `{}` is incompatible with return type `{}`",
                            defname, return_type))
                    }
                } 
            };
        }

        let result = match return_type {
            ReturnType::Bool => {
                chk_defname!(BoolAttributeDefinition::DEFINITION_NAME);
                AttributeDefinition::Bool(BoolAttributeDefinition::new())
            },
            ReturnType::StaticStr => {
                chk_defname!(StaticStrAttributeDefinition::DEFINITION_NAME);
                AttributeDefinition::StaticStr(StaticStrAttributeDefinition::new())
            },
            ReturnType::UnsignedSize => {
                chk_defname!(NumberAttributeDefinition::<usize>::DEFINITION_NAME);
                AttributeDefinition::UnsignedSize(NumberAttributeDefinition::new())
            },
            ReturnType::UnsignedInteger64 => {
                chk_defname!(NumberAttributeDefinition::<u64>::DEFINITION_NAME);
                AttributeDefinition::UnsignedInteger64(NumberAttributeDefinition::new())
            },
            ReturnType::Integer64 => {
                chk_defname!(NumberAttributeDefinition::<i64>::DEFINITION_NAME);
                AttributeDefinition::Integer64(NumberAttributeDefinition::new())
            },
            ReturnType::Float64 => {
                chk_defname!(NumberAttributeDefinition::<f64>::DEFINITION_NAME);
                AttributeDefinition::Float64(NumberAttributeDefinition::new())
            },
            ReturnType::UnsignedInteger32 => {
                chk_defname!(NumberAttributeDefinition::<u32>::DEFINITION_NAME);
                AttributeDefinition::UnsignedInteger32(NumberAttributeDefinition::new())
            },
            ReturnType::Integer32 => {
                chk_defname!(NumberAttributeDefinition::<i32>::DEFINITION_NAME);
                AttributeDefinition::Integer32(NumberAttributeDefinition::new())
            },
            ReturnType::Float32 => {
                chk_defname!(NumberAttributeDefinition::<f32>::DEFINITION_NAME);
                AttributeDefinition::Float32(NumberAttributeDefinition::new())
            },
            ReturnType::Byte => {
                chk_defname!(NumberAttributeDefinition::<u8>::DEFINITION_NAME);
                AttributeDefinition::Byte(NumberAttributeDefinition::new())
            },
            ReturnType::BoxedTrait => {
                chk_defname!(RelationAttributeDefinition::DEFINITION_NAME);
                let id = return_identifier.ok_or("Missing Identifier for ReturnType::BoxedTrait")?;
                match definition_name {
                    Some("Rel") | None => {
                        let mut attr_def = AttributeDefinition::Relation(RelationAttributeDefinition::new(id));
                        let rel_def = attr_def.get_relation_definition_mut();
                        rel_def.dispatch = Some(Dispatch::Dynamic);
                        attr_def
                    },
                    Some(s) => {
                        return Err(format!(
                            "Definition type `{}` is incompatible with return type `{}`",
                            s, return_type)) 
                    } 
                }
            },
            ReturnType::BoxedTraitIterator => {
                chk_defname!(RelationAttributeDefinition::DEFINITION_NAME);
                let id = return_identifier.ok_or("Missing Identifier for ReturnType::BoxedTraitIterator")?;
                match definition_name {
                    Some("Rel") | None => {
                        let mut attr_def = AttributeDefinition::Relation(RelationAttributeDefinition::new(id));
                        let rel_def = attr_def.get_relation_definition_mut();
                        rel_def.dispatch = Some(Dispatch::Dynamic);
                        rel_def.nature = Some(RelationNature::OneToMany);
                        attr_def
                    },
                    Some(s) => {
                        return Err(format!(
                            "Definition type `{}` is incompatible with return type `{}`",
                            s, return_type)) 
                    } 
                }
            },
            ReturnType::AssociatedType => {
                chk_defname!(RelationAttributeDefinition::DEFINITION_NAME);
                let id = return_identifier.ok_or("Missing Identifier for ReturnType::AssociatedType")?;
                let mut reldef = RelationAttributeDefinition::new(id);
                reldef.dispatch = Some(Dispatch::Static);
                AttributeDefinition::Relation(reldef)
            },
            // Enums will never be implied as their type cannot be determined without reflection
            // This is here for posterity
            ReturnType::Enum => {
                chk_defname!(FieldlessEnumAttributeDefinition::DEFINITION_NAME);
                let id = return_identifier.ok_or("Missing Identifier for ReturnType::Enum")?;
                AttributeDefinition::FieldlessEnum(FieldlessEnumAttributeDefinition::new(id))
            },
            // Type is a catch-all for return types that cannot be implied: Enum 
            ReturnType::Type => {
                let id = return_identifier.ok_or("Missing Identifier for ReturnType::Type")?;
                match definition_name {
                    // Enums will never be implied as their type cannot be determined without reflection
                    // This is where Enums will actually be initialized (not the match arm for Enum above)
                    Some(FieldlessEnumAttributeDefinition::DEFINITION_NAME) => {
                        AttributeDefinition::FieldlessEnum(FieldlessEnumAttributeDefinition::new(id))
                    },
                    Some(s) => {
                        return Err(format!(
                            "Unknown definition type `{}` for return signature: {}",
                            s, id)) 
                    },
                    None => {
                        return Err(format!(
                            "Unable to determine implied definition type for return signature: {}",
                            id)) 
                    },
                }
            },
        };

        Ok(result)
    }

    pub fn has_default(&self) -> bool {
        match self {
            AttributeDefinition::Bool(booldef) => booldef.default.is_some(),
            AttributeDefinition::StaticStr(strdef) => strdef.default.is_some(),
            AttributeDefinition::UnsignedSize(numdef) => numdef.default.is_some(),
            AttributeDefinition::UnsignedInteger64(numdef) => numdef.default.is_some(),
            AttributeDefinition::Integer64(numdef) => numdef.default.is_some(),
            AttributeDefinition::Float64(numdef) => numdef.default.is_some(),
            AttributeDefinition::UnsignedInteger32(numdef) => numdef.default.is_some(),
            AttributeDefinition::Integer32(numdef) => numdef.default.is_some(),
            AttributeDefinition::Float32(numdef) => numdef.default.is_some(),
            AttributeDefinition::Byte(numdef) => numdef.default.is_some(),
            AttributeDefinition::FieldlessEnum(typedef) => typedef.default.is_some(),
            AttributeDefinition::Relation(_reldef) => false,
            AttributeDefinition::Type(_typedef) => false,
        }
    }

    pub fn default(&self) -> Option<Value> {
        match self {
            AttributeDefinition::Bool(ref booldef) => match &booldef.default {
                Some(b) => Some(Value::Bool(*b)),
                None => None
            },
            AttributeDefinition::StaticStr(ref strdef) => match &strdef.default {
                Some(s) => Some(Value::StaticStr(s.to_string())),
                None => None
            },
            AttributeDefinition::UnsignedSize(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::UnsignedSize(*n)),
                None => None
            },
            AttributeDefinition::UnsignedInteger64(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::UnsignedInteger64(*n)),
                None => None
            },
            AttributeDefinition::Integer64(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::Integer64(*n)),
                None => None
            },
            AttributeDefinition::Float64(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::Float64(*n)),
                None => None
            },
            AttributeDefinition::UnsignedInteger32(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::UnsignedInteger32(*n)),
                None => None
            },
            AttributeDefinition::Integer32(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::Integer32(*n)),
                None => None
            },
            AttributeDefinition::Float32(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::Float32(*n)),
                None => None
            },
            AttributeDefinition::Byte(ref numdef) => match &numdef.default {
                Some(n) => Some(Value::Byte(*n)),
                None => None
            },
            AttributeDefinition::FieldlessEnum(ref typedef) => match &typedef.default {
                Some(id) => Some(Value::EnumVariant(id.clone())),
                None => None
            },
            AttributeDefinition::Relation(_reldef) => None,
            AttributeDefinition::Type(_reldef) => None,
        }
    }

    pub fn has_preset(&self) -> bool {
        match self {
            AttributeDefinition::Bool(_booldef) => false,
            AttributeDefinition::StaticStr(strdef) => strdef.preset.is_some(),
            AttributeDefinition::UnsignedSize(numdef) => numdef.preset.is_some(),
            AttributeDefinition::UnsignedInteger64(numdef) => numdef.preset.is_some(),
            AttributeDefinition::Integer64(numdef) => numdef.preset.is_some(),
            AttributeDefinition::Float64(numdef) => numdef.preset.is_some(),
            AttributeDefinition::UnsignedInteger32(numdef) => numdef.preset.is_some(),
            AttributeDefinition::Integer32(numdef) => numdef.preset.is_some(),
            AttributeDefinition::Float32(numdef) => numdef.preset.is_some(),
            AttributeDefinition::Byte(numdef) => numdef.preset.is_some(),
            AttributeDefinition::FieldlessEnum(_typedef) => false,
            AttributeDefinition::Relation(_reldef) => false,
            AttributeDefinition::Type(_typedef) => false,
        }
        
    }

    pub fn preset(&self, variant_name: &str, ordinal: usize) -> Option<Value> {
        macro_rules! preset_numdef {
            ($value_variant:path, $num_type:ident, $numdef:ident) => {
               {
                    let preset = match &$numdef.preset { Some(p) => p, None => return None };
                    match preset {
                        NumberPreset::Ordinal => Some($value_variant(ordinal as $num_type)),
                        NumberPreset::Serial => {
                            let start = match $numdef.start { Some(n) => n, None => return None };
                            let increment = match $numdef.increment { Some(n) => n, None => return None };
                            let val = start + (ordinal as $num_type * increment);
                            Some($value_variant(val))
                        }
                    } 
                }
            };
        }

        match self {
            AttributeDefinition::Bool(_booldef) => None,
            AttributeDefinition::StaticStr(ref strdef) => {
                let preset = match &strdef.preset { Some(p) => p, None => return None };
                Some(Value::StaticStr(preset.convert(variant_name)))
            },
            AttributeDefinition::UnsignedSize(ref numdef) => preset_numdef!(Value::UnsignedSize, usize, numdef),
            AttributeDefinition::UnsignedInteger64(ref numdef) => preset_numdef!(Value::UnsignedInteger64, u64, numdef),
            AttributeDefinition::Integer64(ref numdef) => preset_numdef!(Value::Integer64, i64, numdef),
            AttributeDefinition::Float64(ref numdef) => preset_numdef!(Value::Float64, f64, numdef),
            AttributeDefinition::UnsignedInteger32(ref numdef) => preset_numdef!(Value::UnsignedInteger32, u32, numdef),
            AttributeDefinition::Integer32(ref numdef) => preset_numdef!(Value::Integer32, i32, numdef),
            AttributeDefinition::Float32(ref numdef) => preset_numdef!(Value::Float32, f32, numdef),
            AttributeDefinition::Byte(ref numdef) => preset_numdef!(Value::Byte, u8, numdef),
            AttributeDefinition::FieldlessEnum(_typedef) => None,
            AttributeDefinition::Relation(_reldef) => None,
            AttributeDefinition::Type(_typedef) => None,
        }
    }

    pub fn has_default_or_preset(&self) -> bool {
        self.has_default() || self.has_preset()
    }

    pub fn default_or_preset(&self, variant_name: &str, ordinal: usize) -> Option<Value> {
        if self.has_default() {
            self.default()
        } else {
            self.preset(variant_name, ordinal)
        }
    }

    pub fn needs_value(&self) -> bool {
        match self {
            AttributeDefinition::Relation(ref reldef) => match &reldef.nature {
                Some(relationship) => match relationship {
                    RelationNature::OneToOne => false,
                    RelationNature::OneToMany => true,
                    RelationNature::ManyToOne => false,
                },
                None => true,
            },
            _ => true
        }
    }

    /// Ensures that this definition is valid based return type, presets, etc.
    pub fn validate(&self) -> Result<(), &str> {
        if self.has_default() && self.has_preset() {
            return Err("Both a default and a preset have been set");
        }

        match self {
            AttributeDefinition::Bool(_booldef) => Ok(()),
            AttributeDefinition::StaticStr(_strdef) => Ok(()),
            AttributeDefinition::UnsignedSize(numdef) => numdef.validate(),
            AttributeDefinition::UnsignedInteger64(numdef) => numdef.validate(),
            AttributeDefinition::Integer64(numdef) => numdef.validate(),
            AttributeDefinition::Float64(numdef) => numdef.validate(),
            AttributeDefinition::UnsignedInteger32(numdef) => numdef.validate(),
            AttributeDefinition::Integer32(numdef) => numdef.validate(),
            AttributeDefinition::Float32(numdef) => numdef.validate(),
            AttributeDefinition::Byte(numdef) => numdef.validate(),
            AttributeDefinition::FieldlessEnum(enumdef) => enumdef.validate(),
            AttributeDefinition::Relation(reldef) => reldef.validate(),
            AttributeDefinition::Type(_) => unreachable!("Type definitions should not be directly accessible"),
        }
    }

    pub fn get_relation_definition(&self) -> &RelationAttributeDefinition {
        match self {
            Self::Relation(ref def) => def,
            _ => unreachable!("Unexpected definition type: {}", RelationAttributeDefinition::DEFINITION_NAME)
        }
    }

    pub fn get_relation_definition_mut(&mut self) -> &mut RelationAttributeDefinition {
        match self {
            Self::Relation(ref mut def) => def,
            _ => unreachable!("Unexpected definition type: {}", RelationAttributeDefinition::DEFINITION_NAME)
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoolAttributeDefinition {
    pub(crate) default: Option<bool>,
}

impl BoolAttributeDefinition {
    const DEFINITION_NAME: &'static str = "Bool";

    pub fn new() -> Self {
        Self {
            default: None
        }
    }

    pub fn validate(&self) -> Result<(), &str> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumberAttributeDefinition<N> {
    pub(crate) default: Option<N>,
    pub(crate) preset: Option<NumberPreset>,
    pub(crate) start: Option<N>,
    pub(crate) increment: Option<N>,
}

impl<N> NumberAttributeDefinition<N> {
    const DEFINITION_NAME: &'static str = "Num";

    pub fn new() -> Self {
        Self {
            default: None,
            preset: None,
            start: None,
            increment: None
        }
    }
    
    pub fn validate(&self) -> Result<(), &str> {
        let preset = match &self.preset { Some(p) => p, None => return Ok(()) };
        match preset {
            NumberPreset::Ordinal => Ok(()),
            NumberPreset::Serial => {
                if self.start.is_none() {
                    Err("Missing attribute for `Serial` number preset: start")
                } else if self.increment.is_none() {
                    Err("Missing attribute for `Serial` number preset: increment")
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Presets use the variant name as input and output a case conversion using the [convert_case](https://docs.rs/convert_case/latest/convert_case/enum.Case.html)
/// crate. The `Variant` preset does no conversion.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StringPreset {
    /// The unaltered variant name
    Variant,
    /// "my_variant_name"
    Snake, 
    /// "MY_VARIANT_NAME"
    UpperSnake,
    /// "my-variant-name"
    Kebab,
    /// "MY-VARIANT-NAME"
    UpperKebab,
     /// "myVariantName"
    Camel,
    /// "My Variant Name"
    Title,
    /// "MY VARIANT NAME"
    Upper,
    /// "my variant name"
    Lower,
   /// "myvariantname"
    Flat,
    /// "MYVARIANTNAME"
    UpperFlat,
    /// "My-Variable-Name"
    Train
}

impl FromStr for StringPreset {
    type Err = ();

    fn from_str(variant_name: &str) -> Result<Self, Self::Err> {
        match variant_name {
            "Variant" => Ok(Self::Variant),
            "Snake" => Ok(Self::Snake),
            "UpperSnake" => Ok(Self::UpperSnake),
            "Kebab" => Ok(Self::Kebab),
            "UpperKebab" => Ok(Self::UpperKebab),
            "Camel" => Ok(Self::Camel),
            "Title" => Ok(Self::Title),
            "Upper" => Ok(Self::Upper),
            "Lower" => Ok(Self::Lower),
            "Flat" => Ok(Self::Flat),
            "UpperFlat" => Ok(Self::UpperFlat),
            "Train" => Ok(Self::Train),
            _ => Err(())
        }
    }
}

impl StringPreset {
    pub fn convert(&self, text: &str) -> String {
        match self {
            Self::Variant => text.to_owned(),
            Self::Snake => text.to_case(case::Case::Snake),
            Self::UpperSnake => text.to_case(case::Case::ScreamingSnake),
            Self::Kebab => text.to_case(case::Case::Kebab),
            Self::UpperKebab => text.to_case(case::Case::UpperKebab),
            Self::Camel => text.to_case(case::Case::Camel),
            Self::Title => text.to_case(case::Case::Title),
            Self::Upper => text.to_case(case::Case::Upper),
            Self::Lower => text.to_case(case::Case::Lower),
            Self::Flat => text.to_case(case::Case::Flat),
            Self::UpperFlat => text.to_case(case::Case::UpperFlat),
            Self::Train => text.to_case(case::Case::Train),
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NumberPreset {
    Ordinal,
    Serial,
}

impl FromStr for NumberPreset {
    type Err = ();

    fn from_str(variant_name: &str) -> Result<Self, Self::Err> {
        match variant_name {
            "Ordinal" => Ok(Self::Ordinal),
            "Serial" => Ok(Self::Serial),
            _ => Err(())
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaticStrAttributeDefinition {
    pub(crate) default: Option<String>,
    pub(crate) preset: Option<StringPreset>,
}

impl StaticStrAttributeDefinition {
    const DEFINITION_NAME: &'static str = "Str";

    pub fn new() -> Self {
        Self {
            default: None,
            preset: None
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldlessEnumAttributeDefinition {
    identifier: Identifier,
    default: Option<Identifier>
}

impl FieldlessEnumAttributeDefinition {
    const DEFINITION_NAME: &'static str = "Enum";

    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            default: None
        }
    }

    pub fn validate(&self) -> Result<(), &str> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeAttributeDefinition {
    identifier: Identifier,
}

impl TypeAttributeDefinition {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RelationNature {
    OneToOne,
    OneToMany,
    ManyToOne
}

impl FromStr for RelationNature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OnetoOne" => Ok(Self::OneToOne),
            "OneToMany" => Ok(Self::OneToMany),
            "ManyToOne" => Ok(Self::ManyToOne),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Dispatch {
    Static,
    Dynamic
}

impl FromStr for Dispatch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Static" => Ok(Self::Static),
            "Dynamic" => Ok(Self::Dynamic),
            _ => Err(())
        }
    }
}



#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelationAttributeDefinition {
    identifier: Identifier,
    dispatch: Option<Dispatch>,
    pub(crate) nature: Option<RelationNature>,
}

impl RelationAttributeDefinition {
    const DEFINITION_NAME: &'static str = "Rel";

    pub fn identifier(&self) -> &Identifier { &self.identifier }
    pub fn dispatch(&self) -> Option<Dispatch> { self.dispatch }
    pub fn nature(&self) -> Option<RelationNature> { self.nature }

    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            nature: None,
            dispatch: None
        }
    }

    pub fn validate(&self) -> Result<(), &str> {
        match self.dispatch{
            Some(Dispatch::Dynamic) => {},
            Some(Dispatch::Static) => return Err("Static dispatch is currently unimplemented"),
            None => return Err("Missing property for Rel definition: nature")
        }

        match self.nature {
            Some(_) => {},
            None => return Err("Missing property for Rel definition: nature")
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Method {
    name: String,
    return_type: ReturnType,
    attribute_definition: AttributeDefinition
}

impl Method {
    pub fn name(&self) -> &str { &self.name }
    pub fn return_type(&self) -> ReturnType { self.return_type }
    pub fn attribute_definition(&self) -> &AttributeDefinition { &self.attribute_definition }

    pub fn new(name: String, return_type: ReturnType, attribute_definition: AttributeDefinition) -> Self {
        Self {
            name,
            return_type,
            attribute_definition
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AttributeValue {
    value: Value
}

impl AttributeValue {
    pub fn value(&self) -> &Value { &self.value }

    pub fn new(value: Value) -> Self {
        Self {
            value: value
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    Bool(bool),
    StaticStr(String),
    UnsignedInteger64(u64),
    Integer64(i64),
    Float64(f64),
    UnsignedInteger32(u32),
    Integer32(i32),
    Float32(f32),
    UnsignedSize(usize),
    Byte(u8),
    EnumVariant(Identifier),
    Relation(Identifier),
    Type(Identifier),
}

impl EnumTrait {
    pub fn serialize(&self) -> bincode::Result<Vec<u8>>{
        bincode::serialize(self)
    }

    pub fn deserialize(bytes: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(bytes)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TraitEnum {
    identifier: Identifier,
    variants: Vec<Variant>,
    named_relation_enum_ids: HashMap<String, Identifier>
}

pub(crate) struct TraitEnumBuilder {
    identifier: Option<Identifier>,
    variants: Option<Vec<Variant>>,
    named_relation_enum_ids: Option<HashMap<String, Identifier>>
}

impl TraitEnumBuilder {
    pub(crate) fn new() -> Self {
        Self {
            identifier: None,
            variants: None,
            named_relation_enum_ids: None
        }
    }

    pub(crate) fn identifier(&mut self, identifier: Identifier) -> &mut Self {
        self.identifier = Some(identifier);
        self
    }

    pub(crate) fn variant(&mut self, variant: Variant) -> &mut Self {
        if let Some(variants) = &mut self.variants {
            variants.push(variant);
        } else {
            self.variants = Some(vec![variant]);
        }

        self
    }

    pub(crate) fn has_relation_enum(&self, relation_name: &str) -> bool {
        if let Some(named_relation_enum_ids) = &self.named_relation_enum_ids {
            named_relation_enum_ids.contains_key(relation_name)
        } else {
            false
        }
    }

    pub(crate) fn relation_enum(&mut self, relation_name: String, enum_identifier: Identifier) -> &mut Self {
        if let Some(named_relation_enum_ids) = &mut self.named_relation_enum_ids{
            named_relation_enum_ids.insert(relation_name, enum_identifier);
        } else {
            self.named_relation_enum_ids = Some({
                let mut map = HashMap::new();
                map.insert(relation_name, enum_identifier);
                map
            });
        }

        self
    }

    pub(crate) fn build(self) -> TraitEnum {
        let identifier = self.identifier
            .expect("Cannot build TraitEnum without an identifier");
        let variants = self.variants.unwrap_or_else(|| Vec::new() );
        let named_relation_enum_ids = self.named_relation_enum_ids.unwrap_or_else(|| HashMap::new() );

        TraitEnum::new(
            identifier,
            variants,
            named_relation_enum_ids
        )
    }
}

impl TraitEnum {
    pub fn identifier(&self) -> &Identifier { &self.identifier }
    pub fn variants(&self) -> &[Variant] { &self.variants }

    pub fn new(identifier: Identifier, variants: Vec<Variant>, relation_enums: HashMap<String, Identifier>)
            -> Self {
        Self {
            identifier,
            variants,
            named_relation_enum_ids: relation_enums
        }
    }

    pub fn variant(&self, name: &str) -> Option<&Variant> {
        self.variants.iter().find(|v| name == v.name )
    }

    /// Each key matches a method name of the enumtrait, which has been modeled with a relation definition
    /// Each value is the Identifier for the related enum (also implementing enumtrait)
    pub fn relation_enums(&self) -> hash_map::Iter<'_, String, Identifier> {
        self.named_relation_enum_ids.iter()
    }

    pub fn relation_enum_identifier(&self, relation_name: &str) -> Option<&Identifier> {
        self.named_relation_enum_ids.get(relation_name)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Variant {
    name: String,
    named_values: HashMap<String, AttributeValue>
}

impl Variant {
    pub fn name(&self) -> &str { &self.name }

    pub fn values(&self) -> hash_map::Iter<'_, String, AttributeValue>{
        self.named_values.iter()
    }

    pub fn new(name: String, value_map: HashMap<String, AttributeValue>) -> Self {
        Self {
            name,
            named_values: value_map
        }
    }

    pub fn has_value(&self, attribute_name: &str) -> bool {
        self.named_values.contains_key(attribute_name)
    }

    pub fn value(&self, attribute_name: &str) -> Option<&AttributeValue> {
        self.named_values.get(attribute_name)
    }

}

pub(crate) struct VariantBuilder {
    name: Option<String>,
    named_values: Option<HashMap<String, AttributeValue>>
}

impl VariantBuilder {
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            named_values: None
        }
    }

    pub(crate) fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub(crate) fn has_value(&self, attribute_name: &str) -> bool {
        if let Some(named_values) = &self.named_values {
            named_values.contains_key(attribute_name)
        } else {
            false
        }
    }

    pub(crate) fn get_value(&self, attribute_name: &str) -> Option<&AttributeValue> {
        if let Some(named_values) = &self.named_values {
            named_values.get(attribute_name)
        } else {
            None
        }
    }

    pub(crate) fn value(&mut self, attribute_name: String, value: AttributeValue) -> &mut Self {
        if let Some(named_values) = &mut self.named_values {
            named_values.insert(attribute_name, value);
        } else {
            self.named_values = Some({
                let mut map = HashMap::new();
                map.insert(attribute_name, value);
                map
            });
        }

        self
    }

    pub(crate) fn build(self) -> Variant {
        let name = self.name
            .expect("Cannot build Variant without a name");
        let named_values = self.named_values.unwrap_or_else(|| HashMap::new());

        Variant::new(
            name,
            named_values
        )
    }
}

