use serde;

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

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnumVariantIdentifier {
    enum_identifier: Identifier,
    variant: String
}

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
    EnumVariant,
    Relation
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
    EnumVariant(EnumVariantAttributeDefinition),
    Relation(RelationAttributeDefinition)
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NumberAttributeDefinition<N> {
    default: Option<N>,
    start: Option<N>,
    increment: Option<N>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StringFormat {
    format_string: String,
    arguments: Vec<String> //TODO: Vec<Method>?
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StaticStrAttributeDefinition {
    default: Option<String>,
    format: Option<StringFormat>
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnumVariantAttributeDefinition {
    enum_identifier: Identifier,
    default: Option<EnumVariantIdentifier>
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
    EnumVariant(EnumVariantIdentifier),
    Relation(EnumVariantIdentifier)
}

impl From<&'static [u8]> for EnumTrait {
    fn from(bytes: &'static [u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}

