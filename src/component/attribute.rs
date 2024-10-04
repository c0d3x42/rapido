
use super::*;
mod integer;
mod string;

use integer::AttributeInteger;
use string::AttributeString;

#[derive(Debug, Deserialize, Clone)]
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
}
