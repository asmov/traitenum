
struct Identifier {
    path: Vec<String>,
    name: String
}

struct EnumVariantIdentifier {
    enum_identifier: Identifier,
    variant: String
}

struct EnumTrait {
    identifer: Identifier
}

enum ReturnType {
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

enum AttributeDefinition {
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

struct NumberAttributeDefinition<N> {
    default: Option<N>,
    start: Option<N>,
    increment: Option<N>,
}

struct StringFormat {
    format_string: String,
    arguments: Vec<String> //TODO: Vec<Method>?
}

struct StaticStrAttributeDefinition {
    default: Option<String>,
    format: Option<StringFormat>
}

struct EnumVariantAttributeDefinition {
    enum_identifier: Identifier,
    default: Option<EnumVariantIdentifier>
}

enum RelationshipType {
    OneToMany,
    ManyToOne
}

struct RelationAttributeDefinition {
    enumtrait_identifier: Identifier,
    relationship_type: RelationshipType
}

struct Method {
    name: String,
    return_type: ReturnType,
    attribute_definition: AttributeDefinition
}

struct Attribute {
    name: String,
    value: Value
}

type EnumVariantValue = (Identifier, String);

enum Value {
    StaticStr(&'static str),
    UnsignedInteger64(u64),
    Integer64(i64),
    Float64(f64),
    UnsignedInteger32(u32),
    Integer32(i32),
    Float32(f32),
    Byte(u8),
    EnumVariant(EnumVariantValue)
}

