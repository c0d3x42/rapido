use std::collections::HashMap;

use sea_query::{
    ColumnDef, ColumnType, Iden, IntoIden, StringLen, Table, TableCreateStatement,
    TableDropStatement,
};
use serde::{Deserialize, Serialize};

pub mod attribute;
use attribute::Attribute;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CollectionName(String);
impl Iden for CollectionName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "tbl_{}", self.0).unwrap()
    }
}

/// `Component` represents a database table
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct Component {
    /// table name
    #[serde(rename = "collectionName")]
    pub collection_name: CollectionName,

    pub info: Info,
    pub options: Options,

    /// `attributes` are the columns
    pub attributes: Attributes,
}

impl Component {
    /// generate a CREATE TABLE statement
    pub fn into_table_create_statement(&self) -> TableCreateStatement {
        let mut stmt = Table::create();

        stmt.table(self.collection_name.clone().into_iden())
            .if_not_exists();

        for column_attribute in self.attributes.into_column_defs() {
            stmt.col(column_attribute);
        }
        stmt
    }

    /// generate a DROP TABLE statement
    pub fn into_table_drop_statement(&self) -> TableDropStatement {
        Table::drop()
            .table(self.collection_name.clone().into_iden())
            .if_exists()
            .to_owned()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ColName(String);
impl Iden for ColName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).unwrap()
    }
}

/// Collection of all column definitions
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct Attributes(HashMap<ColName, Attribute>);
impl Attributes {
    /// converts `Attributes` into sea_orm::ColumnDef's
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

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct Options {}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct Info {}
