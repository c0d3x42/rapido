use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]

pub struct Common {
    pub requestId: String,
    pub authToken: Option<String>,
    pub api: Api,

    pub responseOptions: ResponseOptions,

    #[serde(flatten)]
    pub action: Action,
    pub debug: Option<bool>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Action {
    CreateTable {
        params: CreateTableParams,
    },
    DeleteTables {
        params: DeleteTablesParams,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseOptions {
    pub dataFormat: DataFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Api {
    CreateTable,
    DeleteTables,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataFormat {
    Objects,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTableAction {
    #[serde(flatten)]
    common: Common,
    params: CreateTableParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTableParams {
    pub tableName: String,
    pub columns: Vec<Column>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteTablesAction {
    #[serde(flatten)]
    common: Common,
    params: DeleteTablesParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteTablesParams {
    tableNames: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub r#type: FieldType,
    pub autoIncrement: Option<bool>,
    pub length: i64,
    pub constraints: Constraints,
    pub comment: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FieldType {
    VarChar,
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Constraints {
    pub primaryKey: Option<bool>,
    pub nullable: Option<bool>
}

#[derive(Debug,Serialize)]
pub struct PartialComponent<'a> {
    pub label: &'a str,
    pub name: &'a str,
    pub component_name: String
}
