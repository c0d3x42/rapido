use column::ColumnName;

use super::*;

#[serde_with::skip_serializing_none]
#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct AlterTableParams {
    pub table_name: TableName,
    pub add_columns: Vec<Column>,
    pub rename_columns: Vec<RenameColumn>,
    pub drop_columns: Vec<ColumnName>
}



#[derive(Debug,Serialize,Deserialize)]
pub struct RenameColumn {
    pub name: ColumnName,
    pub new_name: ColumnName
}