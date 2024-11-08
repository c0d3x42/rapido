use std::fmt::Debug;

use sea_query::Iden;
use serde_json::Value;

use crate::component::ColName;


pub trait Entity: Sync +Debug {

    fn get_all_columns(&self) -> Vec<()>;
}

pub trait Insertable: Sync +Debug {

    fn insert_value( &self, value: Value) -> Vec<(&ColName,String)>;
}