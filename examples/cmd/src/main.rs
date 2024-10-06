use std::{fs::File, io::BufReader};

use command_executor::CommandExecutor;
use component::{ComponentSchema, ParsedComponent};
use database::{SqliteDatabase, SqliteLocalConfig, DB};
use rapido_core::*;
use sql_executor::SqlExecutor;
use sql_generator::SqlGenerator;
use traits::Entity;


#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let file = File::open("component.json").expect("a component file");
    let reader = BufReader::new(file);

    let comp: ComponentSchema = serde_json::from_reader(reader).expect("to parse content");

    println!("COMP {:#?}", comp);

    let parsed :ParsedComponent = comp.into();

    let mut db : DB<SqliteDatabase> = SqliteDatabase::build(SqliteLocalConfig::default()).await.unwrap().into();

    let generator = db.get_generator();

    let tbl  = generator.get_create_table_sql(&parsed);
    println!("TBL = {tbl}");
    db.execute_plain(&tbl).await.unwrap();

}
