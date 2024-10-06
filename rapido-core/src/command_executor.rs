use std::fmt::Debug;

use crate::{sql_executor::SqlExecutor, sql_generator::SqlGenerator};


pub trait CommandExecutor: SqlExecutor +Debug{
    type G: SqlGenerator +Sync+Debug;

    fn get_generator(&self) -> &Self::G;
}