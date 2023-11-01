use serde;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Identifier {
    pub(crate) path: Vec<String>,
    pub(crate) name: String
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct EnumVariantIdentifier {
    enum_identifier: Identifier,
    variant: String
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EnumTrait {
    pub(crate) identifer: Identifier,
    pub(crate) methods: Vec<Method>
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ReturnType {
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
pub(crate) enum AttributeDefinition {
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
pub(crate) struct NumberAttributeDefinition<N> {
    default: Option<N>,
    start: Option<N>,
    increment: Option<N>,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct StringFormat {
    format_string: String,
    arguments: Vec<String> //TODO: Vec<Method>?
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct StaticStrAttributeDefinition {
    default: Option<String>,
    format: Option<StringFormat>
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct EnumVariantAttributeDefinition {
    enum_identifier: Identifier,
    default: Option<EnumVariantIdentifier>
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum RelationshipType {
    OneToMany,
    ManyToOne
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RelationAttributeDefinition {
    enumtrait_identifier: Identifier,
    relationship_type: RelationshipType
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Method {
    pub(crate) name: String,
    pub(crate) return_type: ReturnType,
    pub(crate) attribute_definition: AttributeDefinition
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Attribute {
    name: String,
    value: Value
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum Value {
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

