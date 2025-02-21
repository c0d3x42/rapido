#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use std::sync::Arc;

use axum::{debug_handler, Extension};
use loco_rs::prelude::*;
use migration::{IntoIden, SqliteQueryBuilder};
use rapido_core::{
    command_executor::CommandExecutor,
    component::{ComponentSchema, ParsedComponent, RapidoComponents},
    ddl::TableDefinition,
    seatraits::Executable,
    sql_executor::SqlExecutor,
    sql_generator::SqlGenerator,
    Component, ComponentType,
};
use sea_orm::sea_query::{OnConflict, PostgresQueryBuilder};
use sea_orm::sqlx;
use serde::{Deserialize, Serialize};
use tap::Tap;
use tokio::sync::Mutex;

use crate::{
    app::Dynamic,
    models::_entities::component::{ActiveModel, Column, ComponentWrapper, Entity, Model},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub title: Option<String>,
    pub content: TableDefinition,
}

impl Params {
    fn update(&self, item: &mut ActiveModel) {
        item.title = Set(self.title.clone());
        item.content = Set(ComponentWrapper(self.content.clone()));
        item.name = Set(format!("component:table:{}", self.content.table_name.0));
    }
}

async fn load_item(ctx: &AppContext, id: i32) -> Result<Model> {
    let item = Entity::find_by_id(id).one(&ctx.db).await?;
    item.ok_or_else(|| Error::NotFound)
}

#[debug_handler]
pub async fn list(
    State(ctx): State<AppContext>,
    Extension(rapido_components): Extension<Arc<Mutex<RapidoComponents>>>,
) -> Result<Response> {
    let components = rapido_components.lock().await;

    let names: Vec<String> = components
        .get_all_table_names()
        .iter()
        .map(|n| n.to_string())
        .collect();

    format::json(names)
}

#[debug_handler]
pub async fn add(
    State(ctx): State<AppContext>,
    Extension(_dynamo): Extension<Arc<Dynamic>>,
    Extension(rapido_components): Extension<Arc<Mutex<RapidoComponents>>>,

    Json(params): Json<Params>,
) -> Result<Response> {
    let mut rapido_components = rapido_components.lock().await;

    let component = Component::from(&params.content);
    tracing::debug!("Adding {:?}", component);

    let result = rapido_components
        .add_table(
            params.content.clone(),
            ctx.db.get_postgres_connection_pool().clone(),
        )
        .await;
    if let Err(res) = result {
        return format::json(format!("{res}"));
    }
    tracing::debug!("Add table: {:?}", result);

    let mut item = ActiveModel {
        ..Default::default()
    };

    params.update(&mut item);

    let maybe_inserted = Entity::insert(item)
        .on_conflict(
            OnConflict::column(Column::Name)
                .update_column(Column::Content)
                .to_owned(),
        )
        .tap(|i| {
            tracing::info!("I: {:#?}", i);
        })
        .exec(&ctx.db)
        .await?;
    tracing::info!("Maybe Inserted: {:#?}", maybe_inserted);

    let row_id = maybe_inserted.last_insert_id;

    let e = Entity::find_by_id(row_id)
        .one(&ctx.db)
        .await?
        .map(|row| row.content);

    if let Some(component_wrapper) = e {
        let component = component_wrapper.0;

        let pg_pool = ctx.db.get_postgres_connection_pool();
        //let res = component.create_table(&pg_pool).await.unwrap();

        format::json(format!("notdone"))
    } else {
        format::json(format!("error"))
    }
}

#[debug_handler]
pub async fn update(
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
    Json(params): Json<Params>,
) -> Result<Response> {
    let item = load_item(&ctx, id).await?;

    /*
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
     */

    format::json("not done")
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
        .prefix("api/components")
        .add("/", get(list))
        .add("/", post(add))
        .add("/{id}", get(get_one))
        .add("/{id}", delete(remove))
        .add("/{id}", post(update))
}
