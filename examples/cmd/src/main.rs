use std::{fs::File, io::BufReader};

use command_executor::CommandExecutor;
use component::{ComponentSchema, ParsedComponent};
use database::{SqliteDatabase, SqliteLocalConfig, DB};
use ddl::{column::ColumnConstraints, Action, CreateTableAction};
use rapido_core::*;
use sea_query::{
    Expr, QueryBuilder, QueryStatementBuilder, SchemaBuilder, SimpleExpr, SqliteQueryBuilder,
};
use seatraits::Insertable;
use serde_json::json;
use sql_executor::SqlExecutor;
use sql_generator::SqlGenerator;
use sqlx::sqlite::SqliteQueryResult;
use traits::Entity;

#[tokio::main]
async fn main() {
    let jc = json_api::Common {
        requestId: "1".to_string(),
        authToken: None,
        api: json_api::Api::CreateTable,
        responseOptions: json_api::ResponseOptions {
            dataFormat: json_api::DataFormat::Objects,
        },
        debug: None,
        action: json_api::Action::CreateTable {
            params: json_api::CreateTableParams {
                tableName: "tab1".to_string(),
                columns: vec![json_api::Column {
                    name: "id".to_string(),
                    length: 64,
                    r#type: json_api::FieldType::VarChar,
                    autoIncrement: None,
                    comment: None,
                    constraints: json_api::Constraints {
                        primaryKey: None,
                        nullable: None,
                    },
                }],
            },
        },
    };
    let js = serde_json::to_string_pretty(&jc).unwrap();
    println!("{js}");

    let b: json_api::Common = serde_json::from_str(&js).unwrap();
    println!("b: {:#?}", b);

    let ddl_create = ddl::CreateTableAction {
        common: ddl::common::Common::default(),
        table_definition: ddl::create_table::TableDefinition {
            table_name: ddl::TableName("tbl2".to_string()),
            columns: vec![
                ddl::column::Column {
                    not_null: true,
                    default: None,
                    name: ddl::column::ColumnName("id".to_string()),
                    r#type: ddl::column::ColumnType::Number {
                        length: 64,
                        default_value: Some(0),
                    },
                },
                ddl::column::Column {
                    default: None,
                    not_null: true,
                    name: "ts".into(),
                    r#type: ddl::column::ColumnType::Date {
                        format: ddl::column::DateFormat::YYYYMMDD,
                        default_value: Some(ddl::column::DefaultDate::YYYYMMDD(
                            "19840101".to_string(),
                        )),
                    },
                },
            ],
        },
    };
    let js2 = serde_json::to_string_pretty(&ddl_create).unwrap();
    println!("ddl_create {js2}");
    println!(
        "back {:#?}",
        serde_json::from_str::<ddl::CreateTableAction>(&js2).unwrap()
    );
    let stmt = ddl_create.into_table_create_statement();
    println!("STMT {:#?}", stmt);

    let ddl_cmd = ddl::Command {
        action: Action::Create(ddl_create),
    };
    println!("{}", serde_json::to_string_pretty(&ddl_cmd).unwrap());

    println!("Hello, world!");
    let file = File::open("component.json").expect("a component file");
    let reader = BufReader::new(file);

    let comp: ComponentSchema = serde_json::from_reader(reader).expect("to parse content");

    println!("COMP {:#?}", comp);

    let json_stmt =
        comp.insert_from_json(json!({"email": "blah@", "lop2": "lll", "forename": "vvvv"}));
    println!(
        "json stmt: {}",
        json_stmt.unwrap().to_string(SqliteQueryBuilder)
    );

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
    let stmt = comp
        .into_insert_stmt(&["lop"], vec![Expr::value(3)])
        .unwrap();
    println!("STMT2 {} {:#?}", stmt.to_string(SqliteQueryBuilder), stmt);
    let istmt = stmt.build_any(&*query_builder);

    let parsed: ParsedComponent = comp.into();

    let mut db: DB<SqliteDatabase> = SqliteDatabase::build(SqliteLocalConfig::default())
        .await
        .unwrap()
        .into();
    //db.execute_plain(&create_stmt.to_string(SqliteQueryBuilder)).await.unwrap();
    db.execute_plain(&sql).await.unwrap();

    let generator = db.get_generator();

    let tbl = generator.get_create_table_sql(&parsed);
    println!("TBL = {tbl}");
    db.execute_plain(&tbl).await.unwrap();
}
