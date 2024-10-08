
use super::*;
mod integer;
mod string;

use field::FieldType;
use integer::AttributeInteger;
use string::AttributeString;

///
/// Column types
/// 
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Attribute {
    String(AttributeString),
    Integer(AttributeInteger),
}
impl Attribute {
    pub(super) fn into_column_type(&self) -> ColumnType {
        match self {
            Self::String(AttributeString {
                max_length,
                min_length: _,
            }) => ColumnType::String(StringLen::N(max_length.unwrap_or(128))),
            Self::Integer(AttributeInteger { min: _, max:_ }) => ColumnType::Integer,
        }
    }

    pub fn into_field_type(&self) -> FieldType {
        match self {
            Attribute::String(v) => FieldType::String,
            Attribute::Integer(v) => FieldType::Numeric
        }
    }
}
