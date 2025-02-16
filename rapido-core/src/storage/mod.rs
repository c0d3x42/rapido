use std::{
    fs::{self, File},
    path::Path,
};

use error::StorageError;
use fjall::{PartitionCreateOptions, PartitionHandle};
use sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    SelectColumns,
};
use sea_schema::postgres::def::TableDef;

mod database;
mod file;
mod kv;



#[async_trait]
pub trait ComponentInteraction {
    async fn save(&self, table_def: TableDef) -> Result<(), error::StorageError>;
    /**
     * fetches table definition for the named table
     */
    async fn load(&self, table_name: &str) -> Result<TableDef, error::StorageError>;

    /**
     * retrieves the names of the tables
     */
    async fn index(&self) -> Result<Vec<String>, error::StorageError>;
}



pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum StorageError {
        #[error("not found")]
        NotFound,

        #[error("other unhandled")]
        Unhandled,

        #[error("se/de")]
        Serde(#[from] serde_json::Error),

        #[error("fjall")]
        Fjall(#[from] fjall::Error)
    }
}
