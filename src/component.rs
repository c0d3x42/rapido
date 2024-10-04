use std::collections::HashMap;

use sea_query::{ColumnDef, ColumnType, Iden, IntoIden, StringLen};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    #[serde(rename = "collectionName")]
    pub collection_name: String,

    pub info: Info,
    pub options: Options,

    pub attributes: Attributes,
}

impl Iden for Component {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "tbl_{}", self.collection_name).unwrap()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AttributeInteger {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AttributeString {
    #[serde(rename = "maxLength")]
    max_length: Option<u32>,

    #[serde(rename = "minLength")]
    min_length: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Attribute {
    String(AttributeString),
    Integer(AttributeInteger),
}
impl Attribute {
    fn into_column_type(&self) -> ColumnType {
        match self {
            Self::String(AttributeString {
                max_length,
                min_length,
            }) => ColumnType::String(StringLen::N(max_length.unwrap_or(128))),
            Self::Integer(AttributeInteger { min, max }) => ColumnType::Integer,
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ColName(String);
impl Iden for ColName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).unwrap()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Attributes(HashMap<ColName, Attribute>);
impl Attributes {
    pub fn into_column_defs(&self) -> Vec<ColumnDef> {
        self.0
            .iter()
            .map(|(col_name, col_attribute)| {
                let name = col_name.clone().into_iden();
                let types = col_attribute.into_column_type();

                ColumnDef::new_with_type(name, types)
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Options {}

#[derive(Debug, Deserialize, Clone)]
pub struct Info {}
