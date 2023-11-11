use std::fmt::Display;

use serde;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnumTrait {
    identifier: Identifier,
    methods: Vec<Method>
}

impl EnumTrait {
    pub fn identifier(&self) -> &Identifier { &self.identifier }
    pub fn methods(&self) -> &[Method] { &self.methods }

    pub const fn new(identifier: Identifier, methods: Vec<Method>) -> Self {
        Self {
            identifier,
            methods
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
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.path.join("::"), self.name)
    }
}


#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ReturnType {
    StaticStr,
    UnsignedSize,
    UnsignedInteger64,
    Integer64,
    Float64,
    UnsignedInteger32,
    Integer32,
    Float32,
    Byte,
    Type,
    TypeReference
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AttributeDefinition {
    StaticStr(StaticStrAttributeDefinition),
    UnsignedSize(NumberAttributeDefinition<usize>),
    UnsignedInteger64(NumberAttributeDefinition<u64>),
    Integer64(NumberAttributeDefinition<i64>),
    Float64(NumberAttributeDefinition<f64>),
    UnsignedInteger32(NumberAttributeDefinition<u32>),
    Integer32(NumberAttributeDefinition<i32>),
    Float32(NumberAttributeDefinition<f32>),
    Byte(NumberAttributeDefinition<u8>),
    Type(TypeAttributeDefinition),
    Relation(RelationAttributeDefinition)
}

impl AttributeDefinition {
    pub fn has_default(&self) -> bool {
        match self {
            AttributeDefinition::StaticStr(strdef) => strdef.default.is_some(),
            AttributeDefinition::UnsignedSize(numdef) => numdef.default.is_some(),
            AttributeDefinition::UnsignedInteger64(numdef) => numdef.default.is_some(),
            AttributeDefinition::Integer64(numdef) => numdef.default.is_some(),
            AttributeDefinition::Float64(numdef) => numdef.default.is_some(),
            AttributeDefinition::UnsignedInteger32(numdef) => numdef.default.is_some(),
            AttributeDefinition::Integer32(numdef) => numdef.default.is_some(),
            AttributeDefinition::Float32(numdef) => numdef.default.is_some(),
            AttributeDefinition::Byte(numdef) => numdef.default.is_some(),
            AttributeDefinition::Type(typedef) => typedef.default.is_some(),
            AttributeDefinition::Relation(_reldef) => false,
        }
    }

    pub fn default(&self) -> Option<Value> {
        match self {
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
            AttributeDefinition::Type(ref typedef) => match &typedef.default {
                Some(id) => Some(Value::Type(id.clone())),
                None => None
            },
            AttributeDefinition::Relation(_reldef) => None,
        }
    }

}

impl From<ReturnType> for AttributeDefinition {
    fn from(return_type: ReturnType) -> Self {
        match return_type {
            ReturnType::StaticStr => AttributeDefinition::StaticStr(StaticStrAttributeDefinition { default: None, preset: None }),
            ReturnType::UnsignedSize => AttributeDefinition::UnsignedSize(NumberAttributeDefinition { default: Some(0), preset: None, increment: None }),
            ReturnType::UnsignedInteger64 => AttributeDefinition::UnsignedInteger64(NumberAttributeDefinition { default: Some(0), preset: None, increment: None }),
            ReturnType::Integer64 => AttributeDefinition::Integer64(NumberAttributeDefinition { default: Some(0), preset: None, increment: None }),
            ReturnType::Float64 => AttributeDefinition::Float64(NumberAttributeDefinition { default: Some(0.0), preset: None, increment: None }),
            ReturnType::UnsignedInteger32 => AttributeDefinition::UnsignedInteger32(NumberAttributeDefinition { default: Some(0), preset: None, increment: None }),
            ReturnType::Integer32 => AttributeDefinition::Integer32(NumberAttributeDefinition { default: Some(0), preset: None, increment: None }),
            ReturnType::Float32 => AttributeDefinition::Float32(NumberAttributeDefinition { default: Some(0.0), preset: None, increment: None }),
            ReturnType::Byte => AttributeDefinition::Byte(NumberAttributeDefinition { default: Some(0), preset: None, increment: None }),
            ReturnType::Type => todo!(),
            ReturnType::TypeReference => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumberAttributeDefinition<N> {
    pub(crate) default: Option<N>,
    pub(crate) preset: Option<NumberPreset>,
    pub(crate) increment: Option<N>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StringPreset {
    Variant,
    //Lower,
    //Upper,
    //Snake,
    //Slug
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NumberPreset {
    Ordinal,
    Increment,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaticStrAttributeDefinition {
    pub(crate) default: Option<String>,
    pub(crate) preset: Option<StringPreset>
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeAttributeDefinition {
    identifier: Identifier,
    is_reference: bool,
    default: Option<Identifier>
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RelationshipType {
    OneToMany,
    ManyToOne
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RelationAttributeDefinition {
    enumtrait_identifier: Identifier,
    relationship_type: RelationshipType
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
    StaticStr(String),
    UnsignedInteger64(u64),
    Integer64(i64),
    Float64(f64),
    UnsignedInteger32(u32),
    Integer32(i32),
    Float32(f32),
    UnsignedSize(usize),
    Byte(u8),
    Type(Identifier),
    Relation(Identifier)
}

impl From<&'static [u8]> for EnumTrait {
    fn from(bytes: &'static [u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

