use actix::Addr;
use futures::future::ready;
use futures::TryFutureExt;
use futures::{future::BoxFuture, FutureExt};
use realm::{Api, Application, Load};
use serde_json::Value;
use std::collections::HashMap;

use crate::client_actor::ClientActor;
use crate::{report_api::Report, QueryInfo, ReportError};

pub struct ReportManagerBuilder<'a> {
    apis: HashMap<&'a str, Report<'a>>,
    client: Addr<ClientActor>,
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

pub fn report_builder(client: Addr<ClientActor>, capacity: usize) -> ReportManagerBuilder<'static> {
    ReportManagerBuilder {
        apis: HashMap::with_capacity(capacity),
        client,
    }
}

pub struct ReportManager<'a> {
    apis: HashMap<&'a str, Report<'a>>,
}

impl<'a> Application for ReportManagerBuilder<'a> {
    type Result = ReportManager<'a>;

    fn finish(self) -> Self::Result {
        ReportManager { apis: self.apis }
    }
}

impl<'a> Api for ReportManager<'a> {
    type Input = QueryInfo;
    type Output = BoxFuture<'a, Option<Result<Vec<Value>, String>>>;

    fn handle(&self, msg: Self::Input) -> Self::Output {
        let name = msg.name.clone();
        let ref api = self.apis;
        let api = api.get(&*name);

        api.map(|api| {
            api.handle(msg)
                .map_err(|err| match err {
                    ReportError::PgError(err) => err.to_string(),
                    ReportError::CustomError(err) => err.into(),
                })
                .map(Some)
                .right_future()
        })
        .unwrap_or_else(|| ready(None).left_future())
        .boxed()
    }
}
