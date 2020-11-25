#[actix_rt::test]
async fn lazy_works() {
    use futures::future::Ready;
    use std::sync::Arc;
    use std::time::Instant;

    use actix::Arbiter;
    use actix::System;
    use futures::FutureExt;

    use super::app::Received;

    use super::app::AppBuilder;
    use crate::api;
    use crate::prelude::*;
    use tokio_postgres::*;

    let now = Instant::now();

    fn launch_query(param: i32, client: Arc<Client>) -> Ready<String> {
        ready(format!(
            "Returning {} + 1 = {result}",
            param,
            result = param + 1
        ))
    };
    let builder = AppBuilder::<String>::new(1);
    let webview = builder
        .load(lazy("api/test", |container| async move {
            let client = connect(
                "host=localhost user=syfers password=KHearts358/2 dbname=db2database",
                NoTls,
            )
            .map(|result| {
                let (client, connection) = result.unwrap();
                Arbiter::spawn(connection.then(|_| async {}));
                Arc::new(client)
            });
            let client = client.await;
            println!("Conection spawned");
            api(move |msg| launch_query(msg, client.clone()))
        }))
        .content(Content::Html("Hello World".into()))
        .finish();

    let addr = unsafe { webview.user_data().as_ptr().as_ref() }.unwrap();

    for x in 0..100 {
        let request = request("api/test".into(), &x as &i32);
        let request: Received = serde_json::to_string(&request).unwrap().into();
        addr.send(request).await;
    }

    System::current().stop();

    println!("Exercise took: {:?}", Instant::now().duration_since(now));
}
