// https://github.com/thegenius/luna-orm/tree/main

use std::fmt::Debug;

use sqlx::{
    any::{AnyArguments, AnyRow},
    sqlite::SqliteArguments,
};

pub mod component;
pub mod database;
pub mod error;
pub mod sql_executor;
pub mod sql_generator;
pub mod command_executor;

pub enum DatabaseType {
    Sqlite,
}

pub enum DatabaseArguments<'a> {
    Sqlite(SqliteArguments<'a>),
}
pub mod traits {
    use super::*;

    pub trait Entity: Sync + Debug {
        fn get_table_name(&self) -> &str;
        fn get_insert_fields(&self) -> Vec<String>;
        fn any_arguments_of_insert(&self) -> AnyArguments<'_>;
        fn get_create_columns(&self) -> Vec<(String, String)>;
    }

    pub trait SelectedEntity: Debug {
        fn from_any_row(row: AnyRow) -> Result<Self, sqlx::Error>
        where
            Self: Sized;
    }

    pub trait Primary: Sync + Debug {
        fn get_table_name(&self) -> &'static str;
        fn get_primary_field_names(&self) -> &'static [&'static str];
    }

    pub trait Selection: Sync + Debug {
        fn get_table_name(&self) -> &'static str;
        fn get_selected_fields(&self) -> Vec<String>;
    }
}
