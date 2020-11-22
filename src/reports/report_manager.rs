use futures::{future::BoxFuture, FutureExt};
use realm::{Api, Application, Load};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio_postgres::Client;

use crate::{report_api::Report, QueryInfo, ReportError};

pub struct ReportManagerBuilder<'a> {
    apis: HashMap<&'a str, Report<'a>>,
    client: Arc<Client>,
}

impl<'a> Load<Report<'a>> for ReportManagerBuilder<'a> {
    type Result = BoxFuture<'a, Self>;

    fn load(mut self, mut api: Report<'a>) -> Self::Result {
        let client = self.client.clone();
        async move {
            api.prepare(client).await;
            self.apis.insert(api.name, api);
            self
        }
        .boxed()
    }
}

pub fn report_builder(client: Arc<Client>, capacity: usize) -> ReportManagerBuilder<'static> {
    ReportManagerBuilder {
        apis: HashMap::with_capacity(capacity),
        client,
    }
}

pub struct ReportManager<'a> {
    apis: Arc<HashMap<&'a str, Report<'a>>>,
}

impl<'a> Application for ReportManagerBuilder<'a> {
    type Result = ReportManager<'a>;

    fn finish(self) -> Self::Result {
        ReportManager {
            apis: Arc::new(self.apis),
        }
    }
}

impl<'a> Api for ReportManager<'a> {
    type Message = QueryInfo;
    type Response = BoxFuture<'a, Option<Result<Vec<Value>, String>>>;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        let name = msg.name.clone();
        let api = self.apis.clone();
        async move {
            let api = api.get(&*name);
            match api {
                Some(api) => Some(api.handle(msg).await.map_err(|err| match err {
                    ReportError::PgError(err) => err.to_string(),
                    ReportError::CustomError(err) => err.into(),
                })),
                None => None,
            }
        }
        .boxed()
    }
}
