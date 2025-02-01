use std::{collections::HashMap, iter::Map};

use sea_query::{
    Alias, ColumnDef, ColumnType, Iden, IdenList, InsertStatement, IntoIden, PostgresQueryBuilder,
    Query, SelectStatement, SimpleExpr, SqliteQueryBuilder, StringLen, Table, TableCreateStatement,
    TableDropStatement, Value,
};
use sea_schema::postgres::{
    def::{Schema, TableDef, TableInfo, Type},
    discovery::SchemaDiscovery,
};
use serde::{Deserialize, Serialize};

pub mod attribute;
pub mod field;
use attribute::Attribute;
use serde_json::Value as JsonValue;
use sqlx::{any::AnyArguments, postgres::PgQueryResult, PgPool};

use crate::{
    ddl::{column::Column, TableDefinition},
    error::{self, RapidoError},
    seatraits::{Executable, Insertable},
};

use super::traits::Entity;

#[derive(Debug)]
pub struct RapidoComponents {
    schema: Schema,
}

impl RapidoComponents {
    pub async fn init_from_database(pool: PgPool, schema: &str) -> Self {
        let schema_discovery = SchemaDiscovery::new(pool, schema);
        let mut schema = schema_discovery
            .discover()
            .await
            .expect("to discover tables in schema");

        let tables: Vec<_> = schema
            .tables
            .into_iter()
            .filter(|p| p.info.name.starts_with("rapido_"))
            .collect();
        schema.tables = tables;

        tracing::debug!("discovered schemes {:#?}", schema);
        RapidoComponents { schema }
    }

    pub fn init_from_tables(tables: Vec<TableDef>, schema: &str) -> Self {
        Self {
            schema: Schema {
                schema: schema.to_string(),
                tables,
            },
        }
    }

    /**
     * add a new table to schema
     */
    pub async fn add_table(&mut self, table_definition: TableDefinition, pool: PgPool) ->Result<(), RapidoError> {
        let table_def = table_definition.into_table_def();
        let table_create_stmt = table_def.write();
        let stmt = table_create_stmt.build(PostgresQueryBuilder);
        let result = sqlx::query(&stmt).execute(&pool).await.and_then(|qr| {
                tracing::debug!("create table row count = {}", qr.rows_affected());
                self.schema.tables.push(table_def);
                Ok(())

        });

        result.map_err(|err| err.into())
    }

    pub fn get_table_def(&self, table_name: &str) -> Option<&TableDef> {
        self.schema
            .tables
            .iter()
            .find(|p| p.info.name == format!("rapido_{table_name}"))
    }

    pub fn get_all_table_def(&self) -> Vec<&TableDef> {
        self.schema.tables.iter().map(|f| f).collect()
    }
    pub fn get_all_table_names(&self) -> Vec<&str> {
        self.get_all_table_def()
            .into_iter()
            .map(|f| f.info.name.as_str())
            .collect()
    }
}

pub struct RapidoComponent<'a> {
    table_def: &'a TableDef,
    not_null_column_names: Vec<&'a str>,
    expected_columns: Vec<&'a str>,
    mandatory_columns: Vec<&'a str>,
}

impl<'a> RapidoComponent<'a> {
    pub fn new(table_def: &'a TableDef) -> Self {
        let not_null_column_names: Vec<&str> = table_def
            .columns
            .iter()
            .filter(|c| c.not_null.is_some())
            .map(|c| c.name.as_str())
            .collect();

        let mandatory_columns = table_def
            .columns
            .iter()
            .filter(|c| c.default.is_none() && c.not_null.is_some() && c.generated.is_none())
            .map(|c| c.name.as_str())
            .collect();

        let expected_columns: Vec<&str> = table_def
            .columns
            .iter()
            .filter_map(|c| c.generated.is_none().then(|| c.name.as_str()))
            .collect();

        Self {
            table_def,
            not_null_column_names,
            expected_columns,
            mandatory_columns,
        }
    }

    pub fn get_column_info(
        &self,
        col_name: &str,
    ) -> Option<&sea_schema::postgres::def::ColumnInfo> {
        self.table_def.columns.iter().find(|c| c.name == col_name)
    }

    pub fn get_column_type(&self, col_name: &str) -> Option<&sea_schema::postgres::def::Type> {
        self.get_column_info(col_name).map(|ci| &ci.col_type)
    }

    fn build_insertable(
        &self,
        value_map: &serde_json::Map<String, JsonValue>,
    ) -> Result<Vec<(&str, Value)>, error::RapidoError> {
        let mut col_value: Vec<(&str, Value)> = Vec::with_capacity(self.table_def.columns.len());

        for column in &self.expected_columns {
            let column_info = self
                .get_column_info(&column)
                .ok_or(error::RapidoError::NotImplemented)?;

            let maybe_json_value = value_map.get(*column);
            let val = match column_info.default {
                Some(_) => maybe_json_value,
                None => Some(maybe_json_value.ok_or(RapidoError::NotImplemented)?),
            };

            if let Some(value) = val {
                let v = if value.is_null() && column_info.not_null.is_none() {
                    into_sea_query_null_value(&column_info.col_type)
                } else {
                    into_sea_query_value(&column_info.col_type, value)
                }
                .ok_or(RapidoError::NotImplemented)?;

                col_value.push((column, v));
            }
        }

        Ok(col_value)
    }

    pub async fn insert(
        &self,
        value_map: &serde_json::Map<String, JsonValue>,
        pool: &PgPool,
    ) -> Result<(), RapidoError> {
        tracing::info!("Inserting json...");

        let ins = self.build_insertable(value_map)?;
        let mut insert_stmt = sea_query::Query::insert();
        let table_iden = Alias::new(&self.table_def.info.name);

        let stmt = insert_stmt
            .into_table(table_iden)
            .columns(ins.iter().map(|t| Alias::new(t.0)))
            .values(ins.into_iter().map(|t| SimpleExpr::Value(t.1)))
            .map_err(|_err| RapidoError::NotImplemented)?
            .to_string(PostgresQueryBuilder);

        let _pg_result = sqlx::query(&stmt)
            .execute(pool)
            .await
            .map_err(|err| RapidoError::SqlxError(err))?;
        Ok(())
    }
}

fn into_sea_query_null_value(coltype: &Type) -> Option<Value> {
    match coltype {
        Type::Varchar(_) | Type::Time(_) => Some(sea_query::Value::String(None)),
        Type::BigInt => Some(sea_query::Value::BigInt(None)),
        _ => None,
    }
}

fn into_sea_query_value(coltype: &Type, value: &serde_json::Value) -> Option<Value> {
    let maybe_value: Option<Value> = match coltype {
        Type::Varchar(_) | Type::Text => {
            let q = value
                .as_str()
                .and_then(|s| Some(sea_query::Value::String(Some(Box::new(s.to_owned())))));
            q
        }
        Type::BigInt => value
            .as_i64()
            .and_then(|i| Some(sea_query::Value::BigInt(Some(i)))),
        _ => None,
    };
    maybe_value
}

struct TableInfoIden(TableInfo);

impl Iden for TableInfoIden {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0.name).unwrap()
    }
}

async fn insert_json_to_table(
    json: serde_json::Map<String, JsonValue>,
    table_def: &TableDef,
    pool: PgPool,
) -> Result<(), RapidoError> {
    let out: Vec<(_, _)> = table_def
        .columns
        .iter()
        .filter_map(|column_info| {
            json.get(&column_info.name)
                .and_then(|value| Some((ColName(column_info.name.clone()), value)))
        })
        .collect();

    let mut insert_stmt = sea_query::Query::insert();
    let table_def_iden = TableInfoIden(table_def.info.clone());
    let insert_stmt =
        insert_stmt
            .into_table(table_def_iden.into_iden())
            .columns(out.iter().map(|c| c.0.clone()))
            .values(out.iter().map(|v| {
                SimpleExpr::Value(sea_query::Value::String(Some(Box::new(v.1.to_string()))))
            }))
            .map_err(|_err| RapidoError::NotImplemented)?;
    let stmt = insert_stmt.to_string(PostgresQueryBuilder);
    let rows = sqlx::query(&stmt)
        .execute(&pool)
        .await
        .map_err(|err| RapidoError::SqlxError(err))?;

    Ok(())
}

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
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ComponentSchema {
    /// table name
    #[serde(rename = "collectionName")]
    pub collection_name: CollectionName,

    /// `attributes` are the columns
    pub column_definitions: Vec<Column>,
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

    /*
    pub fn insert_from_json(
        &self,
        value: serde_json::Value,
    ) -> Result<InsertStatement, sea_query::error::Error> {
        let col_values = self.insert_value(&value);

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

     */
    /// generate a CREATE TABLE statement
    /*
    pub fn into_table_create_statement(&self) -> TableCreateStatement {
        let mut stmt = Table::create();

        stmt.table(self.collection_name.clone().into_iden())
            .if_not_exists();

        for column_attribute in self.attributes.into_column_defs() {
            stmt.col(column_attribute);
        }
        stmt
    }
     */

    /// generate a DROP TABLE statement
    pub fn into_table_drop_statement(&self) -> TableDropStatement {
        Table::drop()
            .table(self.collection_name.clone().into_iden())
            .if_exists()
            .to_owned()
    }

    /*
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
     */
}

/*
impl Executable for ComponentSchema {
    async fn create_table(&self, pool: &sqlx::PgPool) -> Result<PgQueryResult, sqlx::error::Error> {
        let stmt = self
            .into_table_create_statement()
            .build(PostgresQueryBuilder);
        let res = sqlx::query(&stmt).execute(pool).await;
        res
    }

    async fn insert_row(&self, pool: &sqlx::PgPool) {}
}

impl Insertable for ComponentSchema {
    fn insert_value<'v>(&self, value: &'v serde_json::Value) -> Vec<(&ColName, &'v String)> {
        let colnames: Vec<_> = self.attributes.colname_iter().collect();
        println!("colnames: {:#?}", colnames);

        if let serde_json::Value::Object(obj) = value {
            let x = self
                .attributes
                .0
                .iter()
                .filter_map(|(col_name, col_attribute)| {
                    if let Some(JsonValue::String(val)) = obj.get(col_name.to_str()) {
                        Some((col_name, val))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(&ColName, &String)>>();
            x
        } else {
            vec![]
        }
    }
}
 */

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
