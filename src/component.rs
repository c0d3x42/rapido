use std::collections::HashMap;

use sea_query::{ColumnType, DynIden, Iden, IdenStatic, IntoIden};
use serde::{Deserialize};

#[derive(Debug, Deserialize, Clone )]
pub enum ColType {
    int32,
    varchar((u32)),
    chars(u32)
}
impl ColType {
    pub fn into_column_type(&self) -> ColumnType{
        match self{
            ColType::chars(x) => ColumnType::Char(Some(*x)),
            ColType::int32 => ColumnType::Integer,
            ColType::varchar(x) => ColumnType::BigInteger
        }
    }
}



#[derive(Debug, Deserialize,Clone )]
pub struct ColumnDefinition {
    pub name: String,
    pub r#type: ColType
}
impl Iden for ColumnDefinition {
    fn unquoted(&self,s: &mut dyn std::fmt::Write) {
        write!(s, "{}", self.name).unwrap()
    }
}

#[derive(Debug, Deserialize,Clone )]
pub struct Component {
    pub collectionName: String,

    pub info: Info,
    pub options: Options,

    pub name: String,
    pub cols: Vec<ColumnDefinition>,

    pub attributes: HashMap<String, Attribute>
}

impl Iden for Component {
    fn unquoted(&self,s: &mut dyn std::fmt::Write) {
        write!(s, "tbl_{}", self.name).unwrap()
    }
}

#[derive(Debug, Deserialize,Clone )]
pub enum Kind {
    String,
    Integer
}

#[derive(Debug, Deserialize,Clone )]
pub struct Attribute {
    r#type: Kind
}

#[derive(Debug, Deserialize,Clone )]
pub struct Options{}

#[derive(Debug, Deserialize,Clone )]
pub struct Info {}