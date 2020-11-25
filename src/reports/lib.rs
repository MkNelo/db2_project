use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt::Display;
use tokio_postgres::Error;

extern crate actix;
extern crate actix_rt;
extern crate realm;
extern crate report_macros;
extern crate tokio;
extern crate tokio_postgres;

pub mod client_actor;
pub mod report_api;
pub mod report_manager;

#[derive(Serialize, Deserialize)]
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

#[actix_rt::test]
async fn report_manager_works() {
    use client_actor::start_client;
    use realm::prelude::*;
    use realm::Load;
    use serde_json::json;
    let client =
        start_client("host = localhost user = syfers password = KHearts358/2 dbname = db2database")
            .await;
}

#[actix_rt::test]
async fn api_reports_works() {
    use realm::prelude::*;
    use report_macros::params;
    use report_manager::report_builder;
    use crate::report_api::report;
    use serde_json::json;
    use client_actor::start_client;

    let webview = builder::<String>(1)
        .load(actor(start_client("host = localhost user = syfers password = KHearts358/2 dbname = db2database").await))
        .load(lazy("api/test", |msg| async move {
            let container = msg.container();
            let client = container.get().unwrap();
            println!("Conection spawned");
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
        }))
        .content(Content::Html("Hello World".into()))
        .finish();

    let addr = unsafe { webview.user_data().as_ptr().as_ref() }.unwrap();

    for x in 0..100 {
        let request = request("api/test".into(), &QueryInfo {
            name: "second/report".into(),
            params: json!([x%3 as i32])
        });
        let request: Received = serde_json::to_string(&request).unwrap().into();
        addr.send(request).await;
    }
}
