#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::{collections::HashMap, sync::Arc};

use axum::{debug_handler, Extension, Json};
use loco_rs::prelude::*;
use rapido_core::component::RapidoComponents;
use sea_orm::sqlx::{
    self, postgres::PgRow, sqlite::SqliteRow, Column, Database, Decode, Row, Sqlite,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::models::_entities::notes::ActiveModel;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.content = Set(self.content.clone());
    }
}


#[debug_handler]
pub async fn insert_one(
    Path(component): Path<String>,
    State(ctx): State<AppContext>,
    Extension(rapido_components): Extension<Arc<Mutex<RapidoComponents>>>,
    Json(value): Json<serde_json::Value>,
) -> Result<Response> {
    let components = rapido_components.lock().await;

    tracing::info!("looking for component: {component}");

    let object_map = value
        .as_object()
        .ok_or(loco_rs::Error::BadRequest("not a map".to_string()))?;
    let pool = ctx.db.get_postgres_connection_pool();

    let component = components
        .get_tabledef_component(&component)
        .ok_or(loco_rs::Error::BadRequest("no component".to_string()))?;
    component
        .insert(object_map, pool)
        .await
        .map_err(|_err| loco_rs::Error::InternalServerError)?;
    format::json(())
}

#[derive(Debug)]
pub enum MyColValue {
    String(String),
    Number(u32),
}

impl<'r> Decode<'r, Sqlite> for MyColValue {
    /*
    fn decode(value: sea_orm::sqlx::sqlite::SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {

        <&'r str>::decode(value);

        Ok(MyColValue::String("".to_string()))

    }

    fn decode(value: <sea_orm::sqlx::Sqlite as sqlx::database::HasValueRef<'r>>::ValueRef) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let r = <&'r str as sqlx::decode::Decode<'r, sqlx::Sqlite>>::decode(value)?;
        Ok(MyColValue::String(r.to_string()))

    }
    */
    fn decode(
        value: <Sqlite as Database>::ValueRef<'r>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let r = <&'r str as sqlx::decode::Decode<'r, sqlx::Sqlite>>::decode(value)?;
        Ok(MyColValue::String(r.to_string()))
    }
}

#[derive(Debug)]
pub struct RowContainer(HashMap<String, String>);
impl sqlx::FromRow<'_, SqliteRow> for RowContainer {
    fn from_row(row: &SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let mut map = HashMap::new();

        for index in 0..row.len() {}

        for col in row.columns() {
            let name = col.name().to_string();
            map.insert(name.clone(), name);
        }
        Ok(Self(map))
    }
}
impl sqlx::FromRow<'_, PgRow> for RowContainer {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        let mut map = HashMap::new();

        for index in 0..row.len() {}

        for col in row.columns() {
            let name = col.name().to_string();
            map.insert(name.clone(), name);
        }
        Ok(Self(map))
    }
}

#[debug_handler]
pub async fn get_one(
    Path((component, id)): Path<(String, i32)>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    tracing::info!("Dynamo Path: {component}, {id}");
    format::json(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/dynamo/{component}")
        //.add("/", get(list).post(insert))
        .add("/", post(insert_one))
        .add("/{id}", get(get_one))
}
