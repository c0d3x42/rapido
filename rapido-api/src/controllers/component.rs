#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::sync::Arc;

use axum::{debug_handler, Extension};
use loco_rs::prelude::*;
use migration::SqliteQueryBuilder;
use rapido_core::{command_executor::CommandExecutor, component::{ComponentSchema, ParsedComponent}, sql_executor::SqlExecutor, sql_generator::SqlGenerator};
use sea_orm::sqlx;
use serde::{Deserialize, Serialize};

use crate::{app::Dynamic, models::_entities::component::{ActiveModel, ComponentWrapper, Entity, Model}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: Option<String>,
    pub content: rapido_core::component::ComponentSchema,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.content = Set(ComponentWrapper(self.content.clone()));
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Entity::find().all(&ctx.db).await?)
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    Extension(dynamo): Extension<Arc<Dynamic>>,

    Json(params): Json<Params>,
) -> Result<Response> {
    let mut item = ActiveModel {
        ..Default::default()
    };
    params.update(&mut item);
    let item = item.insert(&ctx.db).await?;

    let component = item.content.0.clone();
    let parsed :ParsedComponent = component.into();
    let mut db = dynamo.db.lock().await;

    let sql = db.get_generator().get_create_table_sql(&parsed);
    let res = db.execute_plain(&sql).await.unwrap();

    format::json(format!("{:?}", res))
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;

    let sql = item
        .content
        .0
        .clone()
        .into_table_drop_statement()
        .build(SqliteQueryBuilder);

    tracing::info!("DROP: {sql}");
    let query = sqlx::query(&sql);
    let pool = ctx.db.get_sqlite_connection_pool();
    query.execute(pool).await.expect("to drop table");

    let mut item = item.into_active_model();
    params.update(&mut item);
    let item = item.update(&ctx.db).await?;

    let sql = item
        .content
        .0
        .clone()
        .into_table_create_statement()
        .build(SqliteQueryBuilder);
    let query = sqlx::query(&sql);
    query.execute(pool).await.expect("to recreate table");

    format::json(item)
}

#[debug_handler]
pub async fn remove(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    load_item(&ctx, id).await?.delete(&ctx.db).await?;
    format::empty()
}

#[debug_handler]
pub async fn get_one(Path(id): Path<i32>, State(ctx): State<AppContext>) -> Result<Response> {
    format::json(load_item(&ctx, id).await?)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("components")
        .add("/", get(list))
        .add("/", post(add))
        .add("/:id", get(get_one))
        .add("/:id", delete(remove))
        .add("/:id", post(update))
}
