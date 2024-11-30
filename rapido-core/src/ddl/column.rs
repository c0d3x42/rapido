use sea_query::ColumnDef;

use super::*;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    pub name: ColumnName,
    #[serde(flatten)]
    pub r#type: ColumnType,

    #[serde(default)]
    pub constraints: Option<ColumnConstraints>,
    #[serde(default)]
    pub comment: Option<String>,
}
impl Column {
    pub fn into_column_def(&self) -> ColumnDef {
        let mut column_type = ColumnDef::new_with_type(
            self.name.clone().into_iden(),
            self.r#type.into_seaorm_column_type(),
        );
        if let Some(constraints) = &self.constraints{
            if let Some(nullable) =constraints.nullable{
                if nullable{
                    column_type.null();
                }
            }
            if let Some(unique) = constraints.unique {
                if unique{
                    column_type.unique_key();
                }
            }
        }

        column_type
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnName(pub String);
impl From<&str> for ColumnName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
impl Iden for ColumnName {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.0).expect("to convert column name to iden")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ColumnType {
    #[serde(rename_all = "camelCase")]
    VarChar {
        length: u32,
        default_value: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Number {
        length: i32,
        default_value: Option<i32>,
    },
    #[serde(rename_all = "camelCase")]
    Date {
        format: DateFormat,
        default_value: Option<DefaultDate>,
    },
}
impl ColumnType {
    fn into_seaorm_column_type(&self) -> sea_query::ColumnType {
        match self {
            ColumnType::VarChar {
                length,
                default_value,
            } => sea_query::ColumnType::String(sea_query::StringLen::N(*length)),
            ColumnType::Number {
                length,
                default_value,
            } => sea_query::ColumnType::Integer,
            ColumnType::Date {
                format,
                default_value,
            } => sea_query::ColumnType::Date,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DateFormat {
    YYYYMMDD,
    Custom { format: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DefaultDate {
    #[serde(rename_all = "camelCase")]
    Yesterday,
    #[serde(rename_all = "camelCase")]
    Today,
    #[serde(rename_all = "camelCase")]
    Tomorrow,
    YYYYMMDD(String),
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ColumnConstraints {
    pub nullable: Option<bool>,
    pub unique: Option<bool>,
}
