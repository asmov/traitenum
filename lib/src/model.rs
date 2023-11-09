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

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

impl From<ReturnType> for AttributeDefinition {
    fn from(return_type: ReturnType) -> Self {
        match return_type {
            ReturnType::StaticStr => AttributeDefinition::StaticStr(StaticStrAttributeDefinition { default: None, format: None }),
            ReturnType::UnsignedSize => AttributeDefinition::UnsignedSize(NumberAttributeDefinition { default: Some(0), start: None, increment: None }),
            ReturnType::UnsignedInteger64 => todo!(),
            ReturnType::Integer64 => todo!(),
            ReturnType::Float64 => todo!(),
            ReturnType::UnsignedInteger32 => todo!(),
            ReturnType::Integer32 => todo!(),
            ReturnType::Float32 => todo!(),
            ReturnType::Byte => todo!(),
            ReturnType::Type => todo!(),
            ReturnType::TypeReference => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumberAttributeDefinition<N> {
    pub(crate) default: Option<N>,
    pub(crate) start: Option<N>,
    pub(crate) increment: Option<N>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StringFormat {
    format_string: String,
    arguments: Vec<String> //TODO: Vec<Method>?
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaticStrAttributeDefinition {
    pub(crate) default: Option<String>,
    pub(crate) format: Option<StringFormat>
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
    pub fn return_type(&self) -> &ReturnType { &self.return_type }
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
pub struct Attribute {
    name: String,
    value: Value
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
    Byte(u8),
    Type(Identifier),
    Relation(Identifier)
}

impl From<&'static [u8]> for EnumTrait {
    fn from(bytes: &'static [u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

