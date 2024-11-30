use sea_query::{Iden, IntoIden, Table, TableCreateStatement};
use serde::{Deserialize, Serialize};
pub mod common;
pub mod create_table;
use create_table::CreateTableParams;
pub mod column;
use column::Column;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableName(pub String);
impl Iden for TableName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).expect("to convert table name to iden")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTableAction {
    #[serde(flatten)]
    pub common: common::Common,
    pub params: CreateTableParams,
}
impl CreateTableAction {
    pub fn into_table_create_statement(&self) -> TableCreateStatement {
        let mut stmt = Table::create();
        stmt.table(self.params.table_name.clone().into_iden())
            .if_not_exists();
        for column in &self.params.columns {
            stmt.col(column.into_column_def());
        }
        stmt
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Action {
    Create(CreateTableAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    #[serde(flatten)]
    pub action: Action,
}
