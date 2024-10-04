use std::collections::HashMap;

use sea_query::{ColumnDef, ColumnType, DynIden, Iden, IdenStatic, IntoIden};
use serde::Deserialize;



#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    pub collectionName: String,

    pub info: Info,
    pub options: Options,

    pub attributes: Attributes,
}

impl Iden for Component {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "tbl_{}", self.collectionName).unwrap()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum Kind {
    Integer,
    String,
}
impl Kind {
    fn into_column_type(&self) -> ColumnType {
        match self {
            Self::Integer => ColumnType::Integer,
            Self::String => ColumnType::String(sea_query::StringLen::N(128))
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Attribute {
    pub r#type: Kind,
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
                let types = col_attribute.r#type.into_column_type();

                ColumnDef::new_with_type(name, types)
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Options {}

#[derive(Debug, Deserialize, Clone)]
pub struct Info {}
