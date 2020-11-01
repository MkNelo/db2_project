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
        let api = self.as_ref();
        api.handle(msg)
    }
}

impl<S> Api for dyn AsRef<S>
where
    S: Api,
{
    type Message = S::Message;
    type Response = S::Response;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        let api = self.as_ref();
        api.handle(msg)
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

mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use futures::future::Ready;
    use futures::lock::Mutex;
    use serde::Serialize;
    use tokio::spawn;
    use tokio_postgres::NoTls;

    use super::prelude::*;
    use super::webview::container::WVDataContainer;

    fn container_factory() -> WVDataContainer {
        WVDataContainer(Arc::new(Mutex::new(HashMap::new())))
    }

    fn request_body_from_body<T: Serialize>(t: T) -> InvokeBody {
        InvokeBody {
            api_name: "test/test".into(),
            payload: serde_json::to_value(t).unwrap(),
        }
    }

    fn full_request<T: Serialize>(t: T, container: WVDataContainer) -> InvokeRequest {
        InvokeRequest {
            container,
            body: request_body_from_body(t),
        }
    }

    struct ApiS(i32);

    impl Api for ApiS {
        type Message = i32;

        type Response = Ready<i32>;

        fn handle(&self, msg: Self::Message) -> Self::Response {
            ready(msg + 5)
        }
    }

    #[test]
    fn lazy_container_works() {
        let lazy_api = lazy("api/load", |request| {
            let mut container = request.container();
            async move {
                container.get_or_register(|| async {
                    let (client, _) = tokio_postgres::connect("host = localhost user = syfers password = KHearts358/2 dbname = db2database", NoTls).await.unwrap();
                    client
                })
                .then(move |_| {
                    println!("client invoked");
                    ready(ApiS(2))
                }).await
            }
        });
        let executor = tokio().spawner().0;

        let mut spawns = Vec::with_capacity(20);
        let container = container_factory();

        for _ in 0..20 {
            spawns.push(executor.spawn(lazy_api.handle(full_request(2, container.clone()))));
        }

        let results = spawns
            .into_iter()
            .map(|future| executor.block_on(future).unwrap())
            .flat_map(|result| {
                if let ApiResponse::OpResponse {
                    api_name: _,
                    body: ApiResponseBody { payload, error: _ },
                } = result
                {
                    Some::<i32>(serde_json::from_value(payload.unwrap()).unwrap())
                } else {
                    None
                }
            })
            .fold(0, |acc: i32, x| acc + x);

        assert_eq!(results, 133)
    }
}
