use std::{fmt::Debug, ops::{Deref, DerefMut}, str::FromStr};

use sqlx::{
    any::{AnyArguments, AnyConnectOptions, AnyQueryResult}, sqlite::SqliteConnectOptions, AnyPool
};


use crate::{
    command_executor::CommandExecutor, error::RapidoError, sql_executor::SqlExecutor, sql_generator::{DefaultSqlGenerator,SqlGenerator}, traits::Entity
};

pub trait Database : SqlExecutor +Debug {}

pub struct DB<T: Database>(pub T);

impl<T> Deref for DB<T>
where
    T: Database,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for DB<T>
where
    T: Database,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct SqliteLocalConfig {
    db_file: String
}
impl Default for SqliteLocalConfig{
    fn default() -> Self {
        Self { db_file: "db.sqlite".to_string() }
    }
}

#[derive(Debug)]
pub struct SqliteDatabase {
    pool: AnyPool,
    sql_generator: DefaultSqlGenerator,
}

impl SqlExecutor for SqliteDatabase {
    fn get_pool(&self) -> Result<&AnyPool, RapidoError> {
        Ok(&self.pool)
    }
}

impl CommandExecutor for SqliteDatabase {
    type G = DefaultSqlGenerator;

    fn get_generator(&self) -> &Self::G {
        &self.sql_generator
    }
}

impl SqliteDatabase {
    async fn create<'a>(&mut self, entity: &'a mut dyn Entity) -> Result<bool, RapidoError> {
        let sql = self.sql_generator.get_create_sql(entity);
        let args = entity.any_arguments_of_insert();

        let r = self.execute(&sql, args).await?;

        Ok(true)
    }

    async fn execute(
        &mut self,
        stmt: &str,
        args: AnyArguments<'_>,
    ) -> Result<AnyQueryResult, RapidoError> {
        Ok(sqlx::query_with(stmt, args)
            .execute(self.get_pool()?)
            .await?)
    }

    pub async fn build(config: SqliteLocalConfig) -> Result<Self, RapidoError>{

        sqlx::any::install_default_drivers();

        let url = format!("sqlite:{}", config.db_file);
        let any_options = AnyConnectOptions::from_str(&url).unwrap();
        let any_pool = AnyPool::connect_with(any_options).await.unwrap();

        let generator = DefaultSqlGenerator::new();
        let database = SqliteDatabase {pool: any_pool, sql_generator: generator};

        Ok(database)

    }
}

impl Database for SqliteDatabase{ }

impl From<SqliteDatabase> for DB<SqliteDatabase>{
    fn from(value: SqliteDatabase) -> Self {
        Self(value)
    }
}
