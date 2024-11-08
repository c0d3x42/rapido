use std::collections::HashMap;

use sea_query::{
    ColumnDef, ColumnType, Iden, IdenList, InsertStatement, IntoIden, Query, SelectStatement,
    SimpleExpr, SqliteQueryBuilder, StringLen, Table, TableCreateStatement, TableDropStatement,
    Value,
};
use serde::{Deserialize, Serialize};

pub mod attribute;
pub mod field;
use attribute::Attribute;
use serde_json::Value as JsonValue;
use sqlx::any::AnyArguments;

use crate::{error::RapidoError, seatraits::Insertable};

use super::traits::Entity;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CollectionName(pub String);
impl Iden for CollectionName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "tbl_{}", self.0).unwrap()
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub enum ConflictAction {
    Nothing,
    Update(Vec<String>),
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct Upsert {
    name: String,
    targets: Vec<String>,
    actions: ConflictAction,
}

/// `Component` represents a database table
#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct ComponentSchema {
    /// table name
    #[serde(rename = "collectionName")]
    pub collection_name: CollectionName,

    pub info: Info,
    pub options: Options,

    #[serde(default)]
    pub upserts: Vec<Upsert>,

    /// `attributes` are the columns
    pub attributes: Attributes,
}

mod executor {
    use sea_query::{InsertStatement, QueryBuilder, SqliteQueryBuilder};
    use sqlx::{AnyExecutor, AnyPool};

    struct Database<D: QueryBuilder> {
        qb: D,
    }

    async fn insert(stmt: InsertStatement, pool: AnyPool) {
        let q = Database {
            qb: SqliteQueryBuilder,
        };

        let s = stmt.to_string(q.qb);
        let r = sqlx::query(&s).execute(&pool).await;
    }
}

impl ComponentSchema {
    pub fn to_table_iden(&self) -> impl Iden {
        self.collection_name.clone()
    }

    pub fn insert_from_json(
        &self,
        value: serde_json::Value,
    ) -> Result<InsertStatement, sea_query::error::Error> {
        let col_values = self.insert_value(value);

        let mut stmt = sea_query::Query::insert();
        stmt.into_table(self.collection_name.clone().into_iden())
            .columns(col_values.iter().map(|cv| cv.0.clone()))
            .values(col_values.iter().map(|cv| {
                SimpleExpr::Value(sea_query::Value::String(Some(Box::new(cv.1.clone()))))
            }))?;
        Ok(stmt.to_owned())
    }

    pub fn into_insert_stmt(
        &self,
        columns: &[&str],
        values: Vec<SimpleExpr>,
    ) -> Result<InsertStatement, sea_query::error::Error> {
        let cols: Vec<_> = columns
            .iter()
            .filter_map(|colname| self.attributes.get_column_iden(colname))
            .collect();

        let mut stmt = sea_query::Query::insert();

        stmt.into_table(self.collection_name.clone().into_iden())
            .columns(cols)
            .values(values)?;
        Ok(stmt.to_owned())
    }

    pub fn into_upsert_stmt(
        &self,
        upsert_name: &str,
        columns: &[&str],
        values: Vec<SimpleExpr>,
    ) -> InsertStatement {
        let upsert = self.upserts.iter().find(|u| u.name == upsert_name).unwrap();

        for column in columns {}

        let mut stmt = sea_query::Query::insert();
        stmt
    }

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

impl Insertable for ComponentSchema {
    fn insert_value(&self, value: serde_json::Value) -> Vec<(&ColName, String)> {
        //let colnames: Vec<_> = self.attributes.colname_iter().collect();
        if let serde_json::Value::Object(mut obj) = value {
            let x = self
                .attributes
                .0
                .iter()
                .filter_map(|(col_name, col_attribute)| {
                    if let Some(JsonValue::String(val)) = obj.remove(col_name.to_str()) {
                        Some((col_name, val))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(&ColName, String)>>();
            let remaining_keys: Vec<&String> = obj.keys().collect();

            x
        } else {
            vec![]
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ColName(pub String);
impl ColName {
    fn to_str(&self) -> &str {
        &self.0
    }
}
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

    pub fn colname_iter(&self) -> impl Iterator<Item = &ColName> {
        self.0.keys()
    }

    pub fn get_column_iden(&self, name: &str) -> Option<impl Iden> {
        let col_name = ColName(name.into());
        match self.0.contains_key(&col_name) {
            false => None,
            true => Some(col_name),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
enum IndexKind {
    BTree,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
enum ConstraintKind {
    Unique,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
struct ColumnIndex {
    col_name: ColName,
    index: IndexKind,
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
struct ColumnConstraint {
    col_name: ColName,
    constraint: ConstraintKind,
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq, Default)]
pub struct Options {
    #[serde(default)]
    indexes: Vec<ColumnIndex>,
    #[serde(default)]
    constraints: Vec<ColumnConstraint>,
    #[serde(default)]
    has_created_at: bool,
    #[serde(default)]
    has_updated_at: bool,
    #[serde(default)]
    has_changed_at: bool,
    #[serde(default)]
    has_deleted_at: bool,
}

#[derive(Debug, Deserialize, Clone, Serialize, PartialEq, Eq)]
pub struct Info {}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub r#type: field::FieldType,
}
impl Field {
    pub fn to_column_definition(&self) -> (String, String) {
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

    fn get_create_columns(&self) -> Vec<(String, String)> {
        self.fields
            .list
            .iter()
            .map(|field| field.to_column_definition())
            .collect()
    }
}
