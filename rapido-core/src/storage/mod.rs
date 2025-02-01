use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use db::ComponentWrapper;
use sea_orm::{
    prelude::async_trait::async_trait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    SelectColumns,
};

use crate::ddl::TableDefinition;

pub mod db {
    use super::*;

    use sea_orm::{entity::prelude::*, FromJsonQueryResult};
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone, FromJsonQueryResult, Serialize, Deserialize)]
    pub struct ComponentWrapper(pub(crate) TableDefinition);
    impl PartialEq for ComponentWrapper {
        fn eq(&self, other: &Self) -> bool {
            return false;
        }
    }

    #[derive(Debug, Clone, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "component")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,

        pub table_name: String,

        #[sea_orm(column_type = "JsonBinary")]
        pub content: ComponentWrapper,
    }

    impl Model {}

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[async_trait]
pub trait ComponentInteraction {
    async fn save(&self, table_definition: TableDefinition) -> Result<(), error::StorageError>;
    /**
     * fetches table definition for the named table
     */
    async fn load(&self, table_name: &str) -> Result<TableDefinition, error::StorageError>;

    /**
     * retrieves the names of the tables
     */
    async fn index(&self) -> Result<Vec<String>, error::StorageError>;
}

pub struct StorageDatabase<'a> {
    pub db: &'a DatabaseConnection,
}

#[async_trait]
impl<'a> ComponentInteraction for StorageDatabase<'a> {
    async fn save(&self, table_definition: TableDefinition) -> Result<(), error::StorageError> {
        let mut model = db::ActiveModel::default();
        model.content = sea_orm::Set(ComponentWrapper(table_definition));

        let _ = db::Entity::insert(model).exec(self.db).await;
        Ok(())
    }

    async fn load(&self, table_name: &str) -> Result<TableDefinition, error::StorageError> {
        let row = db::Entity::find()
            .filter(db::Column::TableName.eq(table_name))
            .one(self.db)
            .await
            .map_err(|_err| error::StorageError::Unhandled)?
            .ok_or(error::StorageError::NotFound)?;
        Ok(row.content.0)
    }

    async fn index(&self) -> Result<Vec<String>, error::StorageError> {
        let r = db::Entity::find()
            .select_column(db::Column::TableName)
            .all(self.db)
            .await
            .map_err(|_err| error::StorageError::Unhandled)?
            .into_iter()
            .map(|row| row.table_name)
            .collect();
        Ok(r)
    }
}

pub struct StorageFile<'a> {
    pub directory: &'a str,
}

#[async_trait]
impl<'a> ComponentInteraction for StorageFile<'a> {
    async fn save(&self, table_definition: TableDefinition) -> Result<(), error::StorageError> {
        let filename = format!("{}/{}.json", self.directory, table_definition.table_name);
        let path = Path::new(&filename);
        let file = File::create(path).map_err(|_err| error::StorageError::Unhandled)?;

        serde_json::to_writer(file, &table_definition).expect("to write json file");
        Ok(())
    }

    async fn load(&self, table_name: &str) -> Result<TableDefinition, error::StorageError> {
        let filename = format!("{}/{}.json", self.directory, table_name);
        let path = Path::new(&filename);
        let file = File::open(path).map_err(|_err| error::StorageError::Unhandled)?;

        let content: TableDefinition = serde_json::from_reader(file).expect("to read file");
        Ok(content)
    }

    async fn index(&self) -> Result<Vec<String>, error::StorageError> {
        let path = Path::new(self.directory);

        let files: Vec<String> = fs::read_dir(&path)
            .map_err(|_err| error::StorageError::Unhandled)?
            .into_iter()
            .filter_map(|f| f.ok())
            .map(|f| format!("{:?}", f.file_name()))
            .collect();

        Ok(files)
    }
}

pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum StorageError {
        #[error("not found")]
        NotFound,

        #[error("other unhandled")]
        Unhandled,
    }
}
