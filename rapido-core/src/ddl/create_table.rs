use sea_query::ColumnDef;

use super::*;

#[serde_with::skip_serializing_none]
#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CreateTableParams {
    pub table_name: TableName,
    pub columns: Vec<Column>,
    #[serde(default)]
    pub comment: Option<String>
}
impl CreateTableParams {

    fn into_column_defs(&self) -> Vec<ColumnDef>{
        vec![]
    }
}