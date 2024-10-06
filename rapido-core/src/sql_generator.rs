// https://github.com/thegenius/luna-orm/tree/main

use std::fmt::Debug;

use sqlx::{
    any::{AnyArguments, AnyRow},
    sqlite::SqliteArguments,
};

use crate::traits::{Entity, Primary, Selection};

#[derive(Debug)]
pub struct DefaultSqlGenerator {}
impl DefaultSqlGenerator {
    pub fn new() -> Self {
        Self {}
    }
}
impl SqlGenerator for DefaultSqlGenerator {}

pub trait SqlGenerator {
    fn get_wrap_char(&self) -> char {
        '`'
    }

    fn get_placeholder(&self) -> char {
        '?'
    }

    fn get_create_table_sql(&self, entity: &dyn Entity) -> String {
        let table_name = entity.get_table_name();
        let column_definitions :Vec<String> = entity
            .get_create_columns()
            .iter()
            .map(|(col_name, col_type)| {
                format!(
                    "{}{}{} {}",
                    self.get_wrap_char(),
                    col_name,
                    self.get_wrap_char(),
                    col_type
                )
            })
            .collect();
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {}{}{} ({})",
            self.get_wrap_char(),
            table_name,
            self.get_wrap_char(),
            column_definitions.join(",")
        );
        sql
    }

    fn get_select_sql(&self, selection: &dyn Selection, primary: &dyn Primary) -> String {
        let table_name = primary.get_table_name();
        let selected_fields = selection.get_selected_fields();
        let select_clause = Self::wrap_fields(&selected_fields, self.get_wrap_char());

        let sql = format!(
            "SELECT {} FROM {}{}{}",
            select_clause,
            self.get_wrap_char(),
            table_name,
            self.get_wrap_char()
        );
        sql
    }

    fn get_create_sql(&self, entity: &dyn Entity) -> String {
        let table_name = entity.get_table_name();
        let field_names = entity.get_insert_fields();
        let fields = Self::wrap_fields(&field_names, self.get_wrap_char());

        let marks = Self::generate_question_mark_list(&field_names);

        let sql = format!(
            "INSERT INTO {}{}{} ({}) VALUES({})",
            self.get_wrap_char(),
            table_name,
            self.get_wrap_char(),
            fields,
            marks
        );

        sql
    }

    fn wrap_fields(fields: &[String], wrap_char: char) -> String {
        fields
            .iter()
            .map(|f| format!("{wrap_char}{f}{wrap_char}",))
            .collect::<Vec<String>>()
            .join(",")
    }

    fn generate_question_mark_list(fields: &[String]) -> String {
        fields
            .iter()
            .map(|_| "?".to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}
