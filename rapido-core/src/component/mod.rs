use std::collections::HashMap;

use sea_query::{
    ColumnDef, ColumnType, Iden, IntoIden, Query, SelectStatement, SqliteQueryBuilder, StringLen,
    Table, TableCreateStatement, TableDropStatement,
};
use serde::{Deserialize, Serialize};

pub mod attribute;
pub mod field;
use attribute::Attribute;
use sqlx::any::AnyArguments;

use super::traits::Entity;



#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CollectionName(pub String);
impl Iden for CollectionName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "tbl_{}", self.0).unwrap()
    }
}

/// `Component` represents a database table
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct ComponentSchema {
    /// table name
    #[serde(rename = "collectionName")]
    pub collection_name: CollectionName,

    pub info: Info,
    pub options: Options,

    /// `attributes` are the columns
    pub attributes: Attributes,
}

impl ComponentSchema {
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

    pub fn get_all_statement(&self) -> SelectStatement {
        let columns: Vec<_> = self
            .attributes
            .0
            .iter()
            .map(|(col_name, _)| col_name.clone().into_iden())
            .collect();

        let sql = Query::select()
            .columns(columns)
            .from(self.collection_name.clone().into_iden())
            .to_owned();
        sql
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ColName(pub String);
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

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub r#type: field::FieldType,
}
impl Field {
    pub fn to_column_definition(&self) -> (String,String) {
        (self.name.clone(), self.r#type.to_string())
    }
}

#[derive(Debug, Default)]
pub struct Fields {
    pub list: Vec<Field>,
    pub names: Vec<String>,
}
impl Fields {
    fn from(attributes: Attributes) -> Self {
        let names = attributes
            .0
            .iter()
            .map(|(col_name, _attribute)| col_name.0.clone())
            .collect();
        let list = attributes
            .0
            .iter()
            .map(|(col_name, attribute)| Field {
                name: col_name.0.clone(),
                r#type: attribute.into_field_type(),
            })
            .collect();

        Self { list, names }
    }
}

#[derive(Debug)]
pub struct ParsedComponent {
    pub table_name: String,
    pub fields: Fields,
}

impl From<ComponentSchema> for ParsedComponent {
    fn from(value: ComponentSchema) -> Self {
        Self {
            table_name: value.collection_name.0,
            fields: Fields::from(value.attributes),
        }
    }
}

impl ParsedComponent {}

impl Entity for ParsedComponent {
    fn get_table_name(&self) -> &str {
        &self.table_name
    }
    fn get_insert_fields(&self) -> Vec<String> {
        self.fields.names.clone()
    }

    fn any_arguments_of_insert(&self) -> sqlx::any::AnyArguments<'_> {
        AnyArguments::default()
    }

    fn get_create_columns(&self) -> Vec<(String,String)> {
        
        self.fields.list.iter().map(|field| field.to_column_definition()).collect()
        

    }
}
