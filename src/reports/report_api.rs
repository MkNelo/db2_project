use std::sync::Arc;

use futures::prelude::*;
use realm::{
    prelude::{ready, BoxFuture},
    Api,
};
use serde::Serialize;
use serde_json::{to_value, Value};
use tokio_postgres::{Client, Row, Statement, types::ToSql, types::Type};

use super::QueryInfo;
use super::ReportError;

pub struct Report<'a> {
    pub(crate) name: &'a str,
    query: &'a str,
    types: Option<&'a [Type]>,
    client: Option<Arc<Client>>,
    statement: Option<Statement>,
    solver: Arc<dyn (Fn(Row) -> Value) + Send + Sync>,
    solve_params: &'a (dyn Fn(&Value) -> Option<Vec<Box<(dyn ToSql + Sync + Send)>>> + Send + Sync),
}

impl<'a> Report<'a> {
    pub async fn prepare(&mut self, client: Arc<Client>) {
        self.client = Some(client);
        self.statement = match self.types {
            Some(types) => self.client.as_ref().unwrap().prepare_typed(self.query, types).await,
            None => self.client.as_ref().unwrap().prepare(self.query).await,
        }
        .ok();
    }

    pub fn typed(&mut self, types: &'a [Type]) {
        self.types.replace(types);
    }

    pub fn solve_with<Response, F>(self, solver: F) -> Report<'a>
    where
        Response: Serialize + Sync + Send,
        F: Fn(Row) -> Response + Send + Sync + 'static,
    {
        let Report {
            name,
            query,
            types,
            solver: _,
            client,
            statement,
            solve_params,
        } = self;
        Report {
            name,
            query,
            types,
            solver: Arc::new(move |msg| to_value(solver(msg)).unwrap()),
            client,
            statement,
            solve_params,
        }
    }
}

pub fn report<'a, 
               Params: Fn(&Value) -> Option<Vec<Box<(dyn ToSql + Send + Sync)>>> + Send + Sync + 'static>(name: &'a str, query: &'a str, params: &'a Params) -> Report<'a>
{
    Report {
        name,
        query,
        types: None,
        solver: Arc::new(|_| Value::Null),
        client: None,
        statement: None,
        solve_params: params
    }
}

impl<'a> Api for Report<'a> {
    type Message = QueryInfo;
    type Response = BoxFuture<'a, Result<Vec<Value>, ReportError>>;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        let solver = self.solver.clone();
        let params = self.solve_params;
        let statement = self
            .statement
            .clone()
            .ok_or_else(|| ReportError::CustomError("Report statement not initialized"));
        let client = self
            .client
            .as_ref()
            .cloned()
            .ok_or_else(|| ReportError::CustomError("Client not initialized"));
        let params = params(&msg.params).ok_or_else(|| ReportError::CustomError("Parameters for query could not be parsed"));
        async move {
            ready(client)
                .and_then(|client| {
                    ready(statement).and_then(move |statement| async move {
                        let result = ready(params)
                        .and_then(|params| async move {
                            let params = params
                                        .iter()
                                        .map(|ptr| ptr.as_ref() as &(dyn ToSql + Sync))
                                        .collect::<Vec<&(dyn ToSql + Sync)>>();
                            client.query(&statement, &*params).map_err(ReportError::PgError).await
                        }).await;
                        result
                        .map(|rows| {
                            rows.into_iter()
                                .map(|row| (&*solver)(row))
                                .collect::<Vec<Value>>()
                        })
                    })
                })
                .await
        }
        .boxed()
    }
}
