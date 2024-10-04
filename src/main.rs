mod app;
pub mod schema;
use sea_query::PostgresQueryBuilder;
use std::{fs::File, io::BufReader};

pub mod component;
use component::Component;

fn main() {
    println!("Hello, world!");
    let file = File::open("component.json").expect("a component file");
    let reader = BufReader::new(file);

    let comp: Component = serde_json::from_reader(reader).expect("to parse content");

    println!("COMP {:#?}", comp);

    let table = comp.into_table_create_statement();
    table.build(PostgresQueryBuilder);

    println!("TABLE {:?}", table.to_string(PostgresQueryBuilder));
}
