use super::*;

mod db {

    use sea_orm::{entity::prelude::*, FromJsonQueryResult};
    use sea_schema::postgres::def::TableDef;
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone, FromJsonQueryResult, Serialize, Deserialize)]
    pub struct ComponentWrapper(pub(crate) TableDef);
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

pub struct StorageDatabase<'a> {
    pub db: &'a DatabaseConnection,
}

#[async_trait]
impl<'a> ComponentInteraction for StorageDatabase<'a> {
    async fn save(&self, table_def: TableDef) -> Result<(), error::StorageError> {
        let mut model = db::ActiveModel::default();
        model.content = sea_orm::Set(db::ComponentWrapper(table_def));

        let _ = db::Entity::insert(model).exec(self.db).await;
        Ok(())
    }

    async fn load(&self, table_name: &str) -> Result<TableDef, error::StorageError> {
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
