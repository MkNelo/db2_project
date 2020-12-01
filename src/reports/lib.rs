use serde::{Deserialize, Serialize};
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

impl Serialize for ReportError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let message = match self {
            ReportError::PgError(err) => err.to_string(),
            ReportError::CustomError(sr) => (*sr).into()
        };
        serializer.collect_str(&*message)
    }
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
    pub use crate::client_actor::*;
    pub use crate::report_api::*;
    pub use report_macros::*;
}
