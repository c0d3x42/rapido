//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::{entity::prelude::*, FromJsonQueryResult };
use serde::{Deserialize, Serialize};
use rapido_core::component::ComponentSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "component")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: Option<String>,

    #[sea_orm(column_type = "JsonBinary")]
    pub content: ComponentWrapper
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[derive(Debug,Serialize,Deserialize, FromJsonQueryResult,Clone, PartialEq, Eq)]
pub struct ComponentWrapper(pub(crate) ComponentSchema);