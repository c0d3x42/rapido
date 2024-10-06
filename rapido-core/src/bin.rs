mod app;
pub mod schema;
use sea_query::{PostgresQueryBuilder, SqliteQueryBuilder};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::{fs::File, io::BufReader, str::FromStr};

pub mod component;
use component::ComponentSchema;

#[async_std::main]
async fn main() {
    println!("Hello, world!");
    let file = File::open("component.json").expect("a component file");
    let reader = BufReader::new(file);

    let comp: ComponentSchema = serde_json::from_reader(reader).expect("to parse content");

    println!("COMP {:#?}", comp);

    let table = comp.into_table_create_statement();
    table.build(PostgresQueryBuilder);

    println!("TABLE {:?}", table.to_string(PostgresQueryBuilder));

    let sql = table.build(SqliteQueryBuilder);

    let conn = SqliteConnectOptions::from_str("data.db").unwrap().create_if_missing(true);
    let pool = SqlitePool::connect_with(conn).await.expect("sqlite");

    let result = sqlx::query(&sql).execute(&pool).await;


}
