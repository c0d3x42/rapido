mod app;
pub mod component;
pub mod schema;
use std::{fs::File, io::BufReader};
use sea_query::{ColumnDef, Iden, IntoIden, PostgresQueryBuilder, Table};

#[derive(Iden)]
enum Document {
    Table,
    Id,
    Uuid,
    JsonField,
    Timestamp,
    TimestampWithTimeZone,
    Decimal,
    Array,
}

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
