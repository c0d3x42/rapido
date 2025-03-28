use std::{
    fs::{self, File},
    path::Path,
};

use enum_dispatch::enum_dispatch;
use error::StorageError;
use fjall::{PartitionCreateOptions, PartitionHandle};
use sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    SelectColumns,
};
use sea_schema::postgres::def::TableDef;

use crate::Component;

mod database;
mod file;
pub mod kv;

#[async_trait]
#[enum_dispatch(StorageBacking)]
pub trait ComponentInteraction {
    /**
     * stores a `TableDef`
     */
    async fn store(&self, table_def: TableDef) -> Result<(), error::StorageError>;

    async fn store_component(&self, component: Component) -> Result<(), error::StorageError>;
    /**
     * fetches table definition for the named table
     */
    async fn fetch(&self, table_name: &str) -> Result<TableDef, error::StorageError>;

    async fn fetch_component(&self, _name: &str) -> Result<Component, error::StorageError> {
        todo!("fetch component")
    }

    async fn fetch_all(&self) -> Result<Vec<TableDef>, StorageError> {
        let mut table_definitions = vec![];
        for table_name in self.index().await? {
            let tabledef = self.fetch(&table_name).await?;
            table_definitions.push(tabledef);
        }

        Ok(table_definitions)
    }

    async fn fetch_all_components(&self) -> Result<Vec<Component>, StorageError> {
        let mut components: Vec<Component> = vec![];
        for name in self.index_component().await? {
            components.push(self.fetch_component(&name).await?);
        }
        Ok(components)
    }

    /**
     * retrieves the names of the tables
     */
    async fn index(&self) -> Result<Vec<String>, error::StorageError>;

    async fn index_component(&self) -> Result<Vec<String>, StorageError> {
        todo!("fetch index of components")
    }
}

#[enum_dispatch]
#[derive(Debug)]
pub enum StorageBacking {
    Database(database::StorageDatabase),
    File(file::StorageFile),
    Kv(kv::StorageKv),
}

pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum StorageError {
        #[error("not found")]
        NotFound,

        #[error("other unhandled")]
        Unhandled,

        #[error("not yet implemented")]
        NotImplemented,

        #[error("se/de")]
        Serde(#[from] serde_json::Error),

        #[error("fjall")]
        Fjall(#[from] fjall::Error),
    }
}
