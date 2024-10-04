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

    for col in comp.cols{
        //table.col(ColumnDef::new(Document::Id).integer());
        let col_type = col.r#type.into_column_type();
        table.col(ColumnDef::new_with_type(col.into_iden(),col_type));
    }
    
    table.build(PostgresQueryBuilder);
    
    println!("TABLE {:?}", table.to_string(PostgresQueryBuilder));
}
