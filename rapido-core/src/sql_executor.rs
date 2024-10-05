use sqlx::{
    any::{AnyArguments, AnyQueryResult, AnyRow},
    AnyPool,
};

use crate::{error::RapidoError, SelectedEntity};

pub trait SqlExecutor {
    fn get_pool(&self) -> Result<&AnyPool, RapidoError> {
        Err(RapidoError::NotImplemented)
    }

    async fn fetch_one<SE>(&mut self, stmt: &str, args: AnyArguments<'_>) -> Result<SE, RapidoError>
    where
        SE: SelectedEntity + Send + Unpin,
    {
        let query = sqlx::query_with(stmt, args).try_map(|row: AnyRow| SE::from_any_row(row));
        let result_opt: SE = query.fetch_one(self.get_pool()?).await?;
        Ok(result_opt)
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
