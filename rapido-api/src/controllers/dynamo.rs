#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

use axum::{debug_handler, Extension};
use loco_rs::prelude::*;
use migration::SqliteQueryBuilder;
use sea_orm::sqlx::{self, decode, sqlite::SqliteRow, Column, Database, Decode, FromRow, Row, Sqlite};
use serde::{Deserialize, Serialize};

use crate::{
    app::Dynamic,
    models::_entities::notes::{ActiveModel, Entity, Model},
};

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
pub async fn list(
    Path(component): Path<String>,
    Extension(dynamo): Extension<Arc<Dynamic>>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    tracing::info!("Dynamo Path: {component}");

    let comp = dynamo
        .get_component(&component)
        .clone()
        .expect("to find a component");
    let (sql, values) = comp.get_all_statement().build(SqliteQueryBuilder);
    //let query = sqlx::query_as::<(String,String)>(&stmt);
    tracing::debug!("query: {:#?}", sql);

    let pool = ctx.db.get_sqlite_connection_pool();
    //let results = query.fetch_all(pool).await.expect("results");
    //let results = query.execute(pool).await.expect("results");
    //tracing::debug!("{:#?}", results);
    let arguments = sqlx::sqlite::SqliteArguments::default();

    let results = sqlx::query_as_with::<_, RowContainer, _>(&sql, arguments)
        .fetch_all(pool)
        .await
        .expect("row results");
    tracing::debug!("ROWS {:#?}", results);

    format::json(component)
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
    */

    fn decode(value: <sea_orm::sqlx::Sqlite as sqlx::database::HasValueRef<'r>>::ValueRef) -> std::result::Result<Self, sqlx::error::BoxDynError> {
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
        .prefix("/dynamo/:component")
        .add("/", get(list))
        .add("/:id", get(get_one))
}
