use futures::Future;
use std::marker::PhantomData;

extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate tokio;

pub mod context;
pub mod data;
pub mod middleware;
pub mod webview;

pub trait Api {
    type Message;
    type Response;

    fn handle(&self, msg: Self::Message) -> Self::Response;
}

impl<S> Api for Box<S>
where
    S: Api + ?Sized,
{
    type Message = S::Message;
    type Response = S::Response;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        self.as_ref().handle(msg)
    }
}

pub type BoxedApi<Message, Response> = Box<dyn Api<Message = Message, Response = Response>>;

pub trait Application {
    type Result;
    fn finish(self) -> Self::Result;
}

pub trait Load<API: Api> {
    type Result;

    fn load(self, api: API) -> Self::Result;
}

pub struct AdHocApi<F, Fut, M>(F, PhantomData<Fut>, PhantomData<M>);

impl<F, Fut, M> Api for AdHocApi<F, Fut, M>
where
    F: Fn(M) -> Fut,
    Fut: Future,
{
    type Message = M;
    type Response = Fut;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        let ref future = self.0;
        future(msg)
    }
}

pub fn api<F, Fut, M>(handler: F) -> AdHocApi<F, Fut, M>
where
    F: Fn(M) -> Fut,
    Fut: Future,
{
    AdHocApi(handler, PhantomData, PhantomData)
}

pub mod prelude {
    pub use super::context::*;
    pub use super::middleware::*;
    pub use super::webview::middleware::*;
    pub use super::webview::*;
    pub use super::{api, Api, Application, Load};
    pub use futures::{executor::*, future::ready, future::BoxFuture, prelude::*};
    pub use web_view::Content::{self, *};
}
