mod app;
pub mod schema;
use std::{fs::File, io::BufReader};
use sea_query::{IntoIden, PostgresQueryBuilder, Table};

pub mod component;
use component::Component;

fn main() {
    println!("Hello, world!");
    let file = File::open("component.json").expect("a component file");
    let reader = BufReader::new(file);

    let comp : Component = serde_json::from_reader(reader).expect("to parse content");

    println!("COMP {:#?}", comp);

    let mut table = Table::create();
    table.table(comp.clone().into_iden()).if_not_exists();

    for col in comp.attributes.into_column_defs(){
        table.col(col);
    }
    
    table.build(PostgresQueryBuilder);
    
    println!("TABLE {:?}", table.to_string(PostgresQueryBuilder));
}
