use futures::future::pending;
use serde::Serialize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, pin::Pin};
use tokio::sync::RwLock;
use tokio_postgres::Client;
use tokio_postgres::NoTls;
use tokio_postgres::Statement;

use super::container::WVDataContainer;
use crate::prelude::*;

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

pub struct ApiS {
    arc: Arc<Client>,
    count: Arc<AtomicUsize>,
}

impl ApiS {
    pub fn new(client: Arc<Client>) -> Self {
        ApiS {
            arc: client,
            count: Arc::new(From::from(0)),
        }
    }

    async fn respond(atomic: Arc<AtomicUsize>, msg: i32) -> i32 {
        let response = msg + 5;
        let countf = atomic.load(Ordering::Relaxed);
        let start = std::time::Instant::now();
        if countf % 2 == 0 {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
        atomic.fetch_add(1, Ordering::Release);

        println!(
            "Responding at: {:?}, with count: {count}",
            std::time::Instant::now().duration_since(start),
            count = countf
        );
        response
    }
}

impl Api for ApiS {
    type Message = i32;
    type Response = BoxFuture<'static, i32>;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        Self::respond(self.count.clone(), msg).boxed()
    }
}

pub fn lazy_apis(request: &mut InvokeRequest) -> impl Future<Output = ApiS> {
    let mut container = request.container();
    async move {
        container
            .get_or_register(|| async {
                let (client, _) = tokio_postgres::connect(
                    "host = localhost user = syfers password = KHearts358/2 dbname = db2database",
                    NoTls,
                )
                .await
                .unwrap();
                client
            })
            .then(move |client| {
                println!("client invoked");
                ready(ApiS::new(client))
            })
            .await
    }
}

pub fn lazy_api_query(request: &mut InvokeRequest) -> impl Future<Output = ApiQuerier> {
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
        let start = std::time::Instant::now();
        if x % 2 == 0 {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
        let value = arc
            .query(&statement, &[])
            .await
            .unwrap()
            .into_iter()
            .nth(x as usize % 3)
            .unwrap()
            .get::<_, &str>(1)
            .into();
        println!(
            "Responding at: {:?}, with value: {value}, with count: {count}",
            std::time::Instant::now().duration_since(start),
            value = value,
            count = x
        );
        value
    }
}

impl Api for ApiQuerier {
    type Message = i32;

    type Response = BoxFuture<'static, String>;

    fn handle(&self, x: Self::Message) -> Self::Response {
        let clone = self.arc.clone();
        let stat = self.delegate.clone();
        Self::query(x, clone, stat).boxed()
    }
}
