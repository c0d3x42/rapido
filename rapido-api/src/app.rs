use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use axum::Extension;
use loco_rs::{
    app::{AppContext, Hooks},
    bgworker::{BackgroundWorker, Queue},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use rapido_core::{
    component::{CollectionName, RapidoComponents},
    database::{SqliteDatabase, SqliteLocalConfig},
};
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;

use crate::{
    controllers,
    models::{
        self,
        _entities::{notes, users},
    },
    tasks,
    workers::downloader::DownloadWorker,
};

pub struct Dynamic {
    pub counter: usize,
    pub components: Vec<rapido_core::component::ComponentSchema>,

    pub db: Mutex<SqliteDatabase>,
}

impl Dynamic {
    fn names(&self) -> Vec<&str> {
        self.components
            .iter()
            .map(|component| component.collection_name.0.as_str())
            .collect()
    }

    pub(crate) fn get_component(
        &self,
        name: &str,
    ) -> Option<&rapido_core::component::ComponentSchema> {
        tracing::debug!("Components: {:#?}", self.names());
        let comp = self
            .components
            .iter()
            .find(|component| component.collection_name.0 == name);

        comp
    }
}

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::notes::routes())
            .add_route(controllers::component::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::user::routes())
            .add_route(controllers::dynamo::routes())
    }

    async fn after_routes(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        /*
        let items = models::_entities::component::Entity::find()
            .all(&ctx.db)
            .await?;
         */

        let dynamic = Dynamic {
            db: Mutex::new(
                SqliteDatabase::build(SqliteLocalConfig::default())
                    .await
                    .unwrap(),
            ),
            counter: 0,
            components: Default::default()
            /*
                items
                .into_iter()
                .map(|item| {
                    tracing::info!(
                        "Loaded Component: [{}] {}",
                        item.id,
                        item.content.collection_name()
                    );
                    let component = item.content.0;
                    component
                })
                .collect(),
             */
        };
        let thing = Arc::new(dynamic);

        let rapido_components = RapidoComponents::init_from_database(
            ctx.db.get_postgres_connection_pool().clone(),
            "rapido",
        )
        .await;
        let rapido = Arc::new(Mutex::new(rapido_components));
        Ok(router.layer(Extension(thing)).layer(Extension(rapido)))
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed::SeedData);
        // tasks-inject (do not remove)
    }

    async fn connect_workers(ctx: &AppContext, queue: &Queue) -> Result<()> {
        queue.register(DownloadWorker::build(ctx)).await?;
        Ok(())
    }

    async fn truncate(db: &DatabaseConnection) -> Result<()> {
        truncate_table(db, users::Entity).await?;
        truncate_table(db, notes::Entity).await?;
        Ok(())
    }

    async fn seed(db: &DatabaseConnection, base: &Path) -> Result<()> {
        db::seed::<users::ActiveModel>(db, &base.join("users.yaml").display().to_string()).await?;
        db::seed::<notes::ActiveModel>(db, &base.join("notes.yaml").display().to_string()).await?;
        Ok(())
    }
}
