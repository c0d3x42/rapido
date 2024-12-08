use sea_query::ColumnDef;
use sea_schema::postgres::def::{ColumnInfo, NotNull, StringAttr};

use super::*;

#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Column {
    pub name: ColumnName,
    #[serde(flatten)]
    pub r#type: ColumnType,
}
impl Column {
    pub fn into_column_def(&self) -> ColumnDef {
        let column_type = ColumnDef::new_with_type(
            self.name.clone().into_iden(),
            self.r#type.into_seaorm_column_type(),
        );

        column_type
    }
}

impl Into<ColumnInfo> for &Column {
    fn into(self) -> ColumnInfo {
        ColumnInfo {
            name: self.name.0.clone(),
            col_type: self.r#type.clone().into(),
            default: None,
            generated: None,
            not_null: None,
            is_identity: false,
        }
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Into<sea_schema::postgres::def::Type> for ColumnType {
    fn into(self) -> sea_schema::postgres::def::Type {
        match self {
            ColumnType::VarChar {
                length,
                default_value,
            } => sea_schema::postgres::def::Type::Varchar(StringAttr::default()),
            _ => sea_schema::postgres::def::Type::Varchar(StringAttr::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DateFormat {
    YYYYMMDD,
    Custom { format: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
