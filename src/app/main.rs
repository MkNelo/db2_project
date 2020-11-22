extern crate db2_project_reports;
extern crate realm;
extern crate serde;
extern crate serde_json;
extern crate tokio_postgres;

use db2_project_reports::prelude::*;
use realm::prelude::*;
use tokio_postgres::NoTls;

struct ApiS(i32);

impl Api for ApiS {
    type Message = i32;
    type Response = BoxFuture<'static, i32>;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        ready(msg + 5).boxed()
    }
}

fn inline_style(s: &str) -> String {
    format!(r#"<style type="text/css">{}</style>"#, s)
}

fn inline_script(s: &str) -> String {
    format!(r#"<script type="text/javascript">{}</script>"#, s)
}

fn main() {
    env_logger::init();
    dotenv::dotenv().expect(".env file not found");

    let file_content = format!(
        r#"
        <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-themes@^1.0.2/dist/cable/index.min.css">
                {custom_css}
                <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/malihu-custom-scrollbar-plugin/3.1.5/jquery.mCustomScrollbar.min.css">
            </head>
            <body>
                <div id="elm"></div>
                {js}
                <script src="https://code.jquery.com/jquery-3.5.1.slim.min.js" integrity="sha384-DfXdz2htPH0lsSSs5nCTpuj/zy4C+OGpamoFVy38MVBnE+IbbVYUew+OrCXaRkfj" crossorigin="anonymous"></script>
                <script src="https://cdnjs.cloudflare.com/ajax/libs/malihu-custom-scrollbar-plugin/3.1.5/jquery.mCustomScrollbar.concat.min.js"></script>
                <script src="https://cdn.jsdelivr.net/npm/bootstrap@4.5.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-ho+j7jyWK8fNQe+A12Hb8AhRq26LrZ/JpcUGGOn+Y7RsweNrtN/tE3MoK7ZeZDyx" crossorigin="anonymous"></script>
                {init}
            </body>
        </html>
        "#,
        custom_css = inline_style(include_str!("styles/custom.css")),
        js = inline_script(include_str!("elm.js")),
        init = inline_script(include_str!("custom.js"))
    );
    builder(tokio())
        .load(lazy("api_configuration", |cx| {
            let mut container = cx.container();
            async move {
                let client = container.get_or_register(|| async move {
                    let (client, connection) = tokio_postgres::connect(
                        "host = localhost user = syfers password = khearts358/2 dbname = db2database",
                        NoTls,
                    )
                        .await
                        .unwrap();
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("Occured error stablishing connection {}", e);
                        }
                    });
                    client
                }).await;
                report_builder(client, 1)
                    .load(report::<(String, String, i32), _>("report/ranking", "SELECT $1, $2 $3;", params!(String, String, i32))).await
                    .finish()
            }.boxed()
        }))
        .content(Content::Html(file_content))
        .size((1024, 720))
        .finish()
        .unwrap()
        .run()
        .ok();
}
