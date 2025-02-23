// https://github.com/thegenius/luna-orm/tree/main

use std::fmt::{format, Debug};

use ddl::TableDefinition;
use sea_schema::postgres::def::TableDef;
use serde::{Deserialize, Serialize};
use sqlx::{
    any::{AnyArguments, AnyRow},
    sqlite::SqliteArguments,
};

pub mod command_executor;
pub mod component;
pub mod database;
pub mod ddl;
pub mod error;
pub mod json_api;
pub mod seatraits;
pub mod sql_executor;
pub mod sql_generator;
pub mod storage;

#[derive(Debug, strum_macros::Display, Serialize, Deserialize, Clone)]
pub enum ComponentType {
    Table(TableDef),
    View,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Component {
    label: String,
    component_type: ComponentType,
    /// normalized name
    name: String,
}
impl Component {
    pub fn new(label: &str, component_type: ComponentType) -> Component {
        let label = label.to_string();
        let name = Self::normalize_name(&label);
        Component {
            label,
            component_type,
            name,
        }
    }

    /**
     * rapido qualified component name
     */
    pub fn component_name(&self) -> String {
        format!("component:{}:{}", self.component_type, self.name)
    }

    pub fn to_component_table_name(name: &str) -> String {
        format!("component:table:{}", name)
    }

    fn normalize_name(name: &str) -> String {
        let mut name = name.trim().replace(" ", "-");
        name.retain(|c| match c {
            'a'..'z' => true,
            '0'..'9' => true,
            '-' => true,
            '_' => true,
            _ => false,
        });
        name
    }
}

impl From<&TableDefinition> for Component {
    fn from(value: &TableDefinition) -> Self {
        Component::new(
            &value.table_name.0,
            ComponentType::Table(value.into_table_def()),
        )
    }
}

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
