use serde::Deserialize;
use serde_json::Value;
use std::fmt::Display;
use tokio_postgres::Error;

extern crate realm;
extern crate report_macros;
pub mod report_api;
pub mod report_manager;

#[derive(Deserialize)]
pub struct QueryInfo {
    name: String,
    params: Value,
}

#[derive(Debug)]
pub enum ReportError {
    PgError(Error),
    CustomError(&'static str),
}

impl Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = f.debug_struct("ReportError");
        match self {
            ReportError::PgError(ref error) => {
                formatter.field("Error from Postgres", error).finish()
            }
            ReportError::CustomError(ref string) => {
                formatter.field("Error in execution", string).finish()
            }
        }
    }
}

impl std::error::Error for ReportError {}

pub mod prelude {
    pub use crate::report_api::*;
    pub use crate::report_manager::*;
    pub use report_macros::*;
}

#[cfg(tests)]
mod tests {
    use std::sync::Arc;

    use realm::prelude::*;
    use report_macros::params;
    use serde_json::json;
    use tokio_postgres::NoTls;

    use crate::QueryInfo;

    #[test]
    fn query_works() {
        let query = params!(i32, String, i32);
        let val = json!([34, "Hello world", 4]);
        let ref opt = query(&val);

        println!("A ver: {:?}", opt);
        assert!(opt.is_some());
    }

    #[test]
    fn report_works() {
        use super::report_api::report;
        use tokio::spawn;
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let (client, connection) = tokio_postgres::connect(
                "host = localhost user = syfers password = KHearts358/2 dbname = db2database",
                NoTls,
            )
            .await
            .unwrap();
            spawn(async move { connection.await });
            let client = Arc::new(client);
            let mut report = report("first/report", "SELECT $1;", params!(String))
                .solve_with::<String, _>(|row| row.get(0));

            {
                report.prepare(client).await;
            }

            report
                .handle(QueryInfo {
                    name: "first/report",
                    params: json!(["Hello world"]),
                })
                .await
        });
        assert_eq!(
            result
                .expect("Result was not Ok")
                .first()
                .expect("Response not there"),
            &json!("Hello World")
        );
    }
}

#[test]
fn report_manager_works() {
    use crate::report_api::report;
    use realm::prelude::*;
    use realm::Load;
    use report_macros::params;
    use report_manager::report_builder;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::spawn;
    use tokio_postgres::NoTls;
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async move {
        let (client, connection) = tokio_postgres::connect(
            "host = localhost user = syfers password = khearts358/2 dbname = db2database",
            NoTls,
        )
        .await
        .unwrap();
        spawn(async move { connection.await });
        let client = Arc::new(client);
        report_builder(client, 2)
            .load(report::<String, _>(
                "first/report",
                "SELECT $1;",
                params!(String),
            ))
            .await
            .load(report::<i32, _>(
                "second/report",
                "SELECT id FROM dummytable WHERE id = $1",
                params!(i32),
            ))
            .await
            .finish()
            .handle(QueryInfo {
                name: "second/report".into(),
                params: json!([2_i32]),
            })
            .await
    });

    assert_eq!(
        result
            .expect("Report not found")
            .expect("Error processing query"),
        vec![json!(2_i32)]
    )
}
