use actix::prelude::*;
use futures::future::ready;
use serde_json::Value;
use tokio_postgres::types::ToSql;
use tokio_postgres::types::Type;
use tokio_postgres::Transaction;
use tokio_postgres::*;

use actix::dev::*;

#[derive(Message)]
#[rtype(result = "Registered")]
pub(crate) enum Register {
    Transaction(Box<dyn FnMut(&mut Transaction) + Send + Sync>),
    Statement(String, Vec<Type>),
}

#[derive(Message)]
#[rtype(result = "ExecutionResult")]
pub(crate) struct QueryStatement<S, F>(
    pub(crate) S,
    pub(crate) F,
    pub(crate) Vec<Box<dyn ToSql + Send + Sync>>,
);

pub(crate) enum RegisterResponse<A> {
    RegisterStatement(ResponseActFuture<A, Registered>),
    RegisterTransaction(AtomicResponse<A, Registered>),
}

impl<A> RegisterResponse<A>
where
    A: Actor,
{
    pub fn transaction<F>(f: F) -> Self
    where
        F: ActorFuture<Actor = A, Output = Registered> + 'static,
    {
        RegisterResponse::RegisterTransaction(AtomicResponse::new(Box::pin(f)))
    }

    pub fn statement<F>(f: F) -> Self
    where
        F: ActorFuture<Actor = A, Output = Registered> + 'static,
    {
        RegisterResponse::RegisterStatement(Box::pin(f))
    }
}

impl<A> MessageResponse<A, Register> for RegisterResponse<A>
where
    A: Actor,
    A::Context: AsyncContext<A>,
{
    fn handle<R: ResponseChannel<Register>>(self, ctx: &mut A::Context, tx: Option<R>) {
        match self {
            RegisterResponse::RegisterTransaction(trans) => trans.handle(ctx, tx),
            RegisterResponse::RegisterStatement(stat) => stat.handle(ctx, tx),
        }
    }
}

pub(crate) enum Registered {
    AppTransaction,
    AppStatement(Statement),
}

impl Registered {
    pub fn statement(self) -> Option<Statement> {
        match self {
            Registered::AppStatement(x) => Some(x),
            _ => None,
        }
    }
}

pub(crate) struct ExecutionResult(pub(crate) Result<Vec<Value>, Error>);

pub struct ClientActor {
    client: Client,
}
impl Actor for ClientActor {
    type Context = Context<Self>;
}

impl Handler<Register> for ClientActor
where
    Self: 'static,
{
    type Result = RegisterResponse<Self>;

    fn handle(&mut self, msg: Register, _: &mut Self::Context) -> Self::Result {
        match msg {
            Register::Transaction(mut prepare) => {
                let reference: *mut Client = &mut self.client;
                let statement = unsafe { async move { (*reference).transaction().await } }
                    .into_actor(self)
                    .then(move |transaction, _, _| {
                        let mut transaction = transaction.unwrap();
                        prepare(&mut transaction);
                        async move {
                            transaction.commit().await.expect("Transaction Failed");
                            Registered::AppTransaction
                        }
                        .actfuture()
                    });
                RegisterResponse::transaction(statement)
            }
            Register::Statement(query, types) => {
                let reference: *const Client = &self.client;
                let statement =
                    unsafe { async move { (*reference).prepare_typed(&*query, &*types).await } }
                        .into_actor(self)
                        .then(|res, _, _| {
                            ready(Registered::AppStatement(res.unwrap())).actfuture()
                        });
                RegisterResponse::statement(statement)
            }
        }
    }
}

impl<S, T> Handler<QueryStatement<S, T>> for ClientActor
where
    S: ToStatement + 'static,
    T: Fn(Row) -> Value + 'static,
{
    type Result = ResponseActFuture<Self, ExecutionResult>;

    fn handle(
        &mut self,
        QueryStatement(stat, map, params): QueryStatement<S, T>,
        _: &mut Self::Context,
    ) -> Self::Result {
        let client: *const Client = &self.client;
        Box::pin(
            async move {
                let params = params
                    .iter()
                    .map(|ptr| ptr.as_ref() as &(dyn ToSql + Sync))
                    .collect::<Vec<&(dyn ToSql + Sync)>>();
                let res = unsafe { (*client).query(&stat, &*params) }
                    .await
                    .map(|vec| vec.into_iter().map(map).collect::<Vec<Value>>());
                ExecutionResult(res)
            }
            .actfuture(),
        )
    }
}

pub async fn start_client(conn: &'static str) -> Addr<ClientActor> {
    let (client, connection) = tokio_postgres::connect(conn, NoTls).await.unwrap();
    Arbiter::spawn(async move {
        connection.await.ok();
    });
    ClientActor { client }.start()
}
