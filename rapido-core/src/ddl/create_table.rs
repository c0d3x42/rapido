use sea_query::ColumnDef;
use sea_schema::postgres::def::TableInfo;

use super::*;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableDefinition {
    pub table_name: TableName,
    pub columns: Vec<Column>,
}
impl TableDefinition {
    fn into_column_defs(&self) -> Vec<ColumnDef> {
        vec![]
    }

    pub fn into_table_def(&self) -> TableDef {
        TableDef {
            info: TableInfo {
                name: format!(r#"rapido.{}"#, self.table_name.0),
                of_type: None,
            },
            columns: self.columns.iter().map(|column| column.into() ).collect(),
            check_constraints: Default::default(),
            not_null_constraints: Default::default(),
            unique_constraints: Default::default(),
            primary_key_constraints: Default::default(),
            reference_constraints: Default::default(),
            exclusion_constraints: Default::default()
        }
    }
}
