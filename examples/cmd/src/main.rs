use std::{fs::File, io::BufReader};

use command_executor::CommandExecutor;
use component::{ComponentSchema, ParsedComponent};
use seatraits::Insertable;
use database::{SqliteDatabase, SqliteLocalConfig, DB};
use rapido_core::*;
use serde_json::json;
use sql_executor::SqlExecutor;
use sql_generator::SqlGenerator;
use sqlx::sqlite::SqliteQueryResult;
use sea_query::{Expr, QueryBuilder, QueryStatementBuilder, SchemaBuilder, SimpleExpr, SqliteQueryBuilder};
use traits::Entity;


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let file = File::open("component.json").expect("a component file");
    let reader = BufReader::new(file);

    let comp: ComponentSchema = serde_json::from_reader(reader).expect("to parse content");

    println!("COMP {:#?}", comp);

    let json_stmt = comp.insert_from_json(json!({"email": "blah@", "lop2": "lll", "forename": "vvvv"}));
    println!("json stmt: {}", json_stmt.unwrap().to_string(SqliteQueryBuilder));

    let create_stmt = comp.into_table_create_statement();
    println!("STMT0 {}", create_stmt.to_string(SqliteQueryBuilder));
    let schema_builder: Box<dyn SchemaBuilder> = Box::new(SqliteQueryBuilder {});
    let query_builder: Box<dyn QueryBuilder> = Box::new(SqliteQueryBuilder {});

    let sql = create_stmt.build_any(&*schema_builder);
    println!("SQL0 {sql}");

    let stmt_x = comp.get_all_statement().build_any(&*query_builder);
    println!("STMTX0 {}, X1 {:?}", stmt_x.0, stmt_x.1);

    let stmt = comp.get_all_statement().build(SqliteQueryBuilder);
    println!("STMT1 {} {:#?}", stmt.0, stmt.1);
    let stmt = comp.into_insert_stmt(&["lop"], vec![Expr::value(3) ]).unwrap();
    println!("STMT2 {} {:#?}", stmt.to_string(SqliteQueryBuilder), stmt);
    let istmt = stmt.build_any(&*query_builder);

    let parsed :ParsedComponent = comp.into();

    let mut db : DB<SqliteDatabase> = SqliteDatabase::build(SqliteLocalConfig::default()).await.unwrap().into();
    //db.execute_plain(&create_stmt.to_string(SqliteQueryBuilder)).await.unwrap();
    db.execute_plain(&sql).await.unwrap();

    let generator = db.get_generator();

    let tbl  = generator.get_create_table_sql(&parsed);
    println!("TBL = {tbl}");
    db.execute_plain(&tbl).await.unwrap();

}
