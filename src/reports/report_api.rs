use actix::prelude::*;
use futures::prelude::*;
use realm::{
    prelude::{ready, BoxFuture},
    Api,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{to_value, Value};
use serde_postgres::from_row;
use tokio_postgres::{row::Row, types::ToSql, types::Type, Client, Statement};

use crate::client_actor::ClientActor;
use crate::client_actor::QueryStatement;
use crate::client_actor::Register;

use super::QueryInfo;
use super::ReportError;

fn from_rows_to_value<Response>(row: Row) -> Value
where
    Response: Serialize + DeserializeOwned,
{
    to_value(from_row::<Response>(&row).unwrap()).unwrap()
}

pub struct Report<'a> {
    pub(crate) name: &'a str,
    query: &'a str,
    types: Option<&'a [Type]>,
    client: Option<Addr<ClientActor>>,
    statement: Option<Statement>,
    solver: fn(Row) -> Value,
    solve_params: &'a (dyn Fn(&Value) -> Option<Vec<Box<(dyn ToSql + Sync + Send)>>> + Send + Sync),
}

impl<'a> Report<'a> {
    pub async fn prepare(&mut self, client: Addr<ClientActor>) {
        self.client = Some(client);
        self.statement = match self.types {
            Some(types) => self
                .client
                .as_ref()
                .unwrap()
                .send(Register::Statement(
                    self.query.into(),
                    self.types.unwrap().into(),
                ))
                .await
                .unwrap()
                .statement()
                .unwrap(),
            None => self
                .client
                .as_ref()
                .unwrap()
                .send(Register::Statement(self.query.into(), vec![]))
                .await
                .unwrap()
                .statement()
                .unwrap(),
        }
        .into();
    }

    pub fn typed(&mut self, types: &'a [Type]) {
        self.types.replace(types);
    }
}

pub fn report<
    'a,
    Response: DeserializeOwned + Serialize + Sync + Send,
    Params: Fn(&Value) -> Option<Vec<Box<(dyn ToSql + Send + Sync)>>> + Send + Sync + 'a,
>(
    name: &'a str,
    query: &'a str,
    params: &'a Params,
) -> Report<'a> {
    Report {
        name,
        query,
        types: None,
        solver: from_rows_to_value::<Response>,
        client: None,
        statement: None,
        solve_params: params,
    }
}

impl<'a> Api for Report<'a> {
    type Input = QueryInfo;
    type Output = BoxFuture<'a, Result<Vec<Value>, ReportError>>;

    fn handle(&self, msg: Self::Input) -> Self::Output {
        let solver = self.solver;
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
        let params = params(&msg.params)
            .ok_or_else(|| ReportError::CustomError("Parameters for query could not be parsed"));
        async move {
            ready(client)
                .and_then(|client| {
                    ready(statement).and_then(move |statement| async move {
                        let result = ready(params)
                            .and_then(|params| async move {
                                let request = QueryStatement(statement, solver, params);
                                client
                                    .send(request)
                                    .await
                                    .unwrap()
                                    .0
                                    .map_err(|err| ReportError::PgError(err))
                            })
                            .await;
                        result
                    })
                })
                .await
        }
        .boxed()
    }
}
