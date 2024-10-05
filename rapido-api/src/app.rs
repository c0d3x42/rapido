use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use axum::Extension;
use loco_rs::{
    app::{AppContext, Hooks},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    worker::{AppWorker, Processor},
    Result,
};
use migration::Migrator;
use rapido_core::component::CollectionName;
use sea_orm::{DatabaseConnection, EntityTrait};

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
}
impl Dynamic {
    pub(crate) fn get_component(&self, name: &str)-> Option<&rapido_core::component::ComponentSchema> {

        let comp = self.components.iter().find(|component| component.collection_name.0 == name  );

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
            .prefix("/api")
            .add_route(controllers::notes::routes())
            .add_route(controllers::component::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::user::routes())
            .add_route(controllers::dynamo::routes())
    }

    async fn after_routes(router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        let items = models::_entities::component::Entity::find()
            .all(&ctx.db)
            .await?;

        let dynamic = Dynamic {
            counter: 0,
            components: items
                .into_iter()
                .map(|item| {
                    let component = item.content.0;
                    component
                })
                .collect(),
        };
        let thing = Arc::new(dynamic);

        Ok(router.layer(Extension(thing)))
    }

    fn connect_workers<'a>(p: &'a mut Processor, ctx: &'a AppContext) {
        p.register(DownloadWorker::build(ctx));
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed::SeedData);
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
