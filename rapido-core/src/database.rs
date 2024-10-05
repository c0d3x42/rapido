use sqlx::{
    any::{AnyArguments, AnyQueryResult},
    AnyPool,
};

use crate::{
    error::RapidoError, sql_executor::SqlExecutor, DefaultSqlGenerator, Entity, SqlGenerator,
};

pub struct SqliteDatabase {
    pool: AnyPool,
    sql_generator: DefaultSqlGenerator,
}

impl SqlExecutor for SqliteDatabase {
    fn get_pool(&self) -> Result<&AnyPool, RapidoError> {
        Ok(&self.pool)
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
}
