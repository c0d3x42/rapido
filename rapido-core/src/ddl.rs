use std::{fmt::Display};

use alter_table::AlterTableParams;
use sea_query::{Iden, IntoIden, Table, TableAlterStatement, TableCreateStatement};
use sea_schema::{postgres::{def::TableDef, discovery::SchemaDiscovery}, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
pub mod alter_table;
pub mod common;
pub mod create_table;
use create_table::TableDefinition;
pub mod column;
use column::Column;
use sqlx::{PgPool, Pool, Postgres, SqlitePool};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableName(pub String);
impl Iden for TableName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).expect("to convert table name to iden")
    }
}
impl Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTableAction {
    #[serde(flatten)]
    pub common: common::Common,
    pub params: TableDefinition,
}
impl CreateTableAction {
    pub fn into_table_create_statement(&self) -> TableCreateStatement {
        let mut stmt = Table::create();
        stmt.table(self.params.table_name.clone().into_iden())
            .if_not_exists();
        for column in &self.params.columns {
            stmt.col(column.into_column_def());
        }

        //let t = serde_json::to_string_pretty(&stmt);
        //println!("STATEMENT: {t}");
        stmt
    }
}

pub struct CreateTableDiscovery {}
impl CreateTableDiscovery {
    pub async fn discover(schema_name: &str, dbpool: PgPool) -> Vec<TableCreateStatement> {
        let schema_discovery = SchemaDiscovery::new(dbpool, schema_name);
        let schema = schema_discovery.discover().await.unwrap();
        let js: Vec<String> = schema
            .tables
            .iter()
            .map(|t| serde_json::to_string_pretty(t).unwrap())
            .collect();

        let tables = schema
            .tables
            .into_iter()
            .map(|table_def| table_def.write())
            .collect();
        tables
    }
    pub async fn disco_def( dbpool: PgPool) -> Vec<TableDef> {
        let discovery = SchemaDiscovery::new(dbpool, "public");
        let schema = discovery.discover().await.unwrap();
        schema.tables
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlterTableAction {
    #[serde(flatten)]
    pub common: common::Common,

    pub params: AlterTableParams,
}
impl AlterTableAction {
    pub fn into_table_alter_statement(&self) -> TableAlterStatement {
        let mut stmt = Table::alter();

        stmt.table(self.params.table_name.clone().into_iden());

        for add_column in &self.params.add_columns {
            stmt.add_column(add_column.into_column_def());
        }

        for rename_column in &self.params.rename_columns {
            stmt.rename_column(
                rename_column.name.clone().into_iden(),
                rename_column.new_name.clone().into_iden(),
            );
        }

        for drop_column in &self.params.drop_columns {
            stmt.drop_column(drop_column.clone().into_iden());
        }

        stmt
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Action {
    Create(CreateTableAction),
    Alter(AlterTableAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    #[serde(flatten)]
    pub action: Action,
}
