use std::collections::HashMap;

use sea_query::{ColumnDef, ColumnType, DynIden, Iden, IdenStatic, IntoIden, StringLen};
use serde::Deserialize;



#[derive(Debug, Deserialize, Clone)]
pub struct Component {
    #[serde(rename="collectionName")]
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


#[derive(Debug,Deserialize, Clone)]
#[serde(tag="type", rename_all="lowercase")]
pub enum Attribute {
    String { maxLength: Option<u32>, minLength: Option<u32>},
    Integer { min: Option<u32>, max: Option<u32>}
}
impl Attribute {
    fn into_column_type(&self) -> ColumnType {
        match self {
            Self::String { maxLength, minLength } => {
                ColumnType::String(StringLen::N(maxLength.unwrap_or(128)))
            },
            Self::Integer { min, max } =>{
                ColumnType::Integer
            }
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
