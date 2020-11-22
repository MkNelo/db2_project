use super::container::WVDataContainer;
use crate::prelude::*;
use crate::prelude::*;
use futures::future::{join_all, pending};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tokio::{runtime::Runtime, spawn, sync::RwLock, time::Instant};

pub fn container_factory() -> WVDataContainer {
    WVDataContainer(Arc::new(RwLock::new(HashMap::new())))
}

pub fn request_body_from_body<T: Serialize>(t: T) -> InvokeBody {
    InvokeBody {
        api_name: "test/test".into(),
        payload: serde_json::to_value(t).unwrap(),
    }
}

pub fn full_request<T: Serialize>(t: T, container: WVDataContainer) -> InvokeRequest {
    InvokeRequest {
        container,
        body: request_body_from_body(t),
    }
}

#[test]
fn lazy_api_query_works() {
    use std::time::Duration;
    pub struct ApiQuerier {
        arc: Arc<Client>,
        delegate: Statement,
    }

    impl ApiQuerier {
        pub async fn new(arc: Arc<Client>) -> Self {
            ApiQuerier {
                delegate: arc.prepare("SELECT * FROM DummyTable").await.unwrap(),
                arc,
            }
        }

        async fn query(x: i32, arc: Arc<Client>, statement: Statement) -> String {
            let value = arc
                .query(&statement, &[])
                .await
                .unwrap()
                .into_iter()
                .nth(x as usize % 3)
                .unwrap()
                .get::<_, &str>(1)
                .into();
            value
        }
    }

    impl Api for ApiQuerier {
        type Message = i32;

        type Response = BoxFuture<'static, String>;

        fn handle(&self, x: Self::Message) -> Self::Response {
            let clone = self.arc.clone();
            let stat = self.delegate.clone();
            ApiQuerier::query(x, clone, stat).boxed()
        }
    }
    use tokio_postgres::{Client, NoTls, Statement};
    fn lazy_api_query(request: &mut InvokeRequest) -> impl Future<Output = ApiQuerier> {
        let mut container = request.container();
        async move {
            container
                .get_or_register(|| async {
                    let (client, conn) = tokio_postgres::connect(
                    "host = localhost user = syfers password = KHearts358/2 dbname = db2database",
                    NoTls,
                )
                .await
                .unwrap();
                    tokio::spawn(async move {
                        println!("Conexión establecida");
                        if let Err(e) = conn.await {
                            eprintln!("Connection error: {}", e)
                        }
                    });
                    client
                })
                .then(move |client| {
                    println!("client invoked");
                    ApiQuerier::new(client)
                })
                .await
        }
    }

    let lazy_api = Arc::new(lazy("api/load", lazy_api_query));
    let mut executor = Runtime::new().unwrap();
    let mut spawns = Vec::with_capacity(20);

    let container = container_factory();

    for x in 0..100 {
        let container_clone = container.clone();
        let clone = lazy_api.clone();
        spawns.push(executor.spawn(async move {
            let request = full_request(x, container_clone);
            let inst = Instant::now();
            let clone = clone.clone();
            clone.handle(request).await;
            println!(
                "Invocation took: {:?} for {}",
                Instant::now().duration_since(inst),
                x
            );
        }));
    }

    executor.block_on(join_all(spawns));
}
