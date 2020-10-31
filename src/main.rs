extern crate realm;
extern crate serde;
extern crate serde_json;
extern crate tokio_postgres;

use std::sync::Arc;
use std::sync::Mutex;

use realm::prelude::*;
use tokio_postgres::Client;
use tokio_postgres::NoTls;

async fn add2(msg: i32) -> i32 {
    msg + 2
}

fn log_invoke(
    ib: InvokeRequest,
    cb: &dyn Fn(InvokeRequest) -> BoxFuture<'static, ApiResponse>,
) -> BoxFuture<'static, ApiResponse> {
    use log::info;
    info!(
        "Got: {} with payload: {}",
        ib.body().api_name(),
        ib.body().payload()
    );
    cb(ib)
        .then(|response| {
            ready({
                info!("Responding with {}", response.body());
                response
            })
        })
        .boxed()
}

struct ApiS(i32);

impl Api for ApiS {
    type Message = i32;
    type Response = BoxFuture<'static, i32>;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        ready(msg + 5).boxed()
    }
}

fn main() {
    env_logger::init();
    dotenv::dotenv().expect(".env file not found");

    let file_content = std::fs::read_to_string("index.html").expect("Html file not found");
    builder(tokio())
        .load(lazy("api/load", |request: &mut InvokeRequest| {
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
        }))
        .load(webview_api("api/add2", api(add2)))
        .load_middleware(
            webview_middleware(log_invoke)
                .invoke_for(vec!["api/add2".into()], |req| req.body().api_name())
        )
        .title("New application")
        .content(Content::Html(file_content))
        .size((1024, 720))
        .finish()
        .unwrap()
        .run()
        .ok();
}
