use std::fmt::Debug;

use sea_query::Iden;
use serde_json::Value;
use sqlx::postgres::PgQueryResult;

use crate::component::ColName;


pub trait Entity: Sync +Debug {

    fn get_all_columns(&self) -> Vec<()>;
}

pub trait Insertable: Sync +Debug {

    /// finds json map keys,values corresponding to columns
    fn insert_value<'v>( &self, value: &'v Value) -> Vec<(&ColName,&'v String)>;
}

pub trait Executable: Sync+Debug{

    async fn create_table(&self, pool: &sqlx::PgPool) -> Result<PgQueryResult, sqlx::error::Error>;

    async fn insert_row(&self, pool: &sqlx::PgPool);
}