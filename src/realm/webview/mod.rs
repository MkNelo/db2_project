use self::api::WVLazyApi;
use self::container::WVDataContainer;
use self::{
    api::WebViewApi,
    container::{WVContainer, WVPreContainer},
};
use super::context::SpawnerFactory;
use super::middleware::none;
use super::middleware::Middleware;
use super::{Api, Application, BoxedApi};
use futures::{future::BoxFuture, task::Spawn, Future, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};
use web_view::{Content, WVResult, WebView};

mod api;
pub(crate) mod container;
pub mod middleware;
#[cfg(tests)]
mod tests;

#[derive(Deserialize)]
pub struct InvokeBody {
    pub(crate) api_name: String,
    pub(crate) payload: Value,
}

pub struct InvokeRequest {
    pub(crate) body: InvokeBody,
    pub(crate) container: WVDataContainer,
}

impl InvokeRequest {
    pub fn container(&self) -> WVDataContainer {
        self.container.clone()
    }

    pub fn body(&self) -> &InvokeBody {
        &self.body
    }
}

impl InvokeBody {
    pub fn api_name(&self) -> &String {
        &self.api_name
    }

    pub fn payload(&self) -> &'_ Value {
        &self.payload
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse {
    OpResponse {
        api_name: String,
        #[serde(flatten)]
        body: ApiResponseBody,
    },
    OpDoNothing(String),
}

impl ApiResponse {
    pub fn body(&self) -> String {
        match self {
            ApiResponse::OpDoNothing(ref name) => format!("[api_name = {}]", name),
            ApiResponse::OpResponse {
                api_name: _,
                body:
                    ApiResponseBody {
                        ref payload,
                        error: _,
                    },
            } => serde_json::to_string(payload).unwrap(),
        }
    }
}

#[derive(Serialize)]
pub struct ApiResponseBody {
    pub(crate) error: bool,
    pub(crate) payload: Option<Value>,
}

pub trait Response<'a>
where
    Self: Future + Send,
    Self::Output: Serialize + Send,
{
    fn into_response(self) -> BoxFuture<'a, Option<ApiResponseBody>>;
}

pub trait Message: for<'de> Deserialize<'de> {
    fn from_message(m: &InvokeRequest) -> Option<Self>;
}

impl<T> Message for T
where
    T: for<'de> Deserialize<'de>,
{
    fn from_message(m: &InvokeRequest) -> Option<Self> {
        serde_json::from_value(m.body().payload().clone()).ok()
    }
}

impl<'a, T> Response<'a> for T
where
    T: Future + std::marker::Send + 'a,
    T::Output: Serialize + Send + 'a,
{
    fn into_response(self) -> BoxFuture<'a, Option<ApiResponseBody>> {
        self.then(|future| async {
            let value = serde_json::to_value(future).unwrap();
            match &value {
                Value::Object(map) if map.contains_key("Ok") => ApiResponseBody {
                    error: false,
                    payload: value.into(),
                }
                .into(),
                Value::Object(map) if map.contains_key("Err") => ApiResponseBody {
                    error: true,
                    payload: value.into(),
                }
                .into(),
                Value::Null => None,
                _ => ApiResponseBody {
                    error: false,
                    payload: value.into(),
                }
                .into(),
            }
        })
        .boxed()
    }
}

pub(crate) type ApiContainer<'a> =
    HashMap<&'a str, BoxedApi<InvokeRequest, BoxFuture<'a, ApiResponse>>>;

pub(crate) type DataContainer = HashMap<TypeId, Arc<dyn Any + Send + Sync + 'static>>;

pub struct WebViewApp<
    'a,
    Cont,
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'a, ApiResponse>> + 'a,
    Context: SpawnerFactory,
> {
    container: WVPreContainer<'a, Mid>,
    webview_builder: web_view::WebViewBuilder<
        'a,
        WVContainer<'a, Mid, Context::Spawner>,
        Box<dyn FnMut(&mut WebView<WVContainer<'a, Mid, Context::Spawner>>, &str) -> WVResult + 'a>,
        Cont,
    >,
    context: Context,
    data_container: DataContainer,
}

impl<'a, Cont, Mid, Context> WebViewApp<'a, Cont, Mid, Context>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'a, ApiResponse>> + 'a,
    Cont: AsRef<str>,
    Context: SpawnerFactory,
{
    pub fn content(mut self, content: Content<Cont>) -> Self {
        self.webview_builder = self.webview_builder.content(content);

        self
    }

    pub fn title<'b: 'a>(mut self, title: &'b str) -> Self {
        self.webview_builder = self.webview_builder.title(title);

        self
    }

    pub fn frameless(mut self, value: bool) -> Self {
        self.webview_builder = self.webview_builder.frameless(value);

        self
    }

    pub fn size(mut self, (width, height): (usize, usize)) -> Self {
        self.webview_builder = self.webview_builder.size(width as i32, height as i32);
        self
    }
}

pub fn webview_api<API>(api_key: &'static str, api: API) -> WebViewApi<API> {
    WebViewApi { api, api_key }
}

pub fn lazy<Factory, Fut, API>(key: &'static str, f: Factory) -> WVLazyApi<Factory, API>
where
    Factory: Fn(&mut InvokeRequest) -> Fut,
    Fut: Future<Output = API>,
{
    WVLazyApi::new(f, key)
}

#[derive(Debug)]
struct NoneError(String);

impl Display for NoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn message_processing<Mid, Spawner>(
    webview: &mut WebView<WVContainer<'static, Mid, Spawner>>,
    arg: &str,
) -> WVResult
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>> + 'static,
    Spawner: Spawn + 'static,
{
    serde_json::from_str(arg)
        .and_then(|invoke_body: InvokeBody| {
            let api_name = invoke_body.api_name.clone();
            let container = webview.user_data();
            let shared_webview = webview.handle();
            let request = InvokeRequest {
                body: invoke_body,
                container: container.data(),
            };

            if let Some(api) = container.get(api_name) {
                let computation =
                    container
                        .solve_with_middleware(request, api)
                        .then(move |response| {
                            futures::future::lazy(move |_| match response {
                                ApiResponse::OpDoNothing(_) => {}
                                x => {
                                    let json_response = serde_json::to_string(&x).unwrap();

                                    shared_webview
                                        .dispatch(move |wv| {
                                            wv.eval(&format!("sendToElm({})", &json_response))
                                        })
                                        .ok();
                                }
                            })
                        });

                container.dispatch(computation).ok();
            }

            Ok(())
        })
        .map_err(|ejson| web_view::Error::custom(ejson))
}

impl<Cont, Mid, Context> Application for WebViewApp<'static, Cont, Mid, Context>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>> + Send,
    Cont: AsRef<str>,
    Context: SpawnerFactory + 'static,
{
    type Result =
        Result<WebView<'static, WVContainer<'static, Mid, Context::Spawner>>, web_view::Error>;

    fn finish(mut self) -> Self::Result {
        let message_callback = Box::new(message_processing);
        self.webview_builder = self
            .webview_builder
            .user_data(self.container.resolve_middleware(
                self.context.spawner(),
                self.data_container,
            ))
            .invoke_handler(message_callback);

        self.webview_builder.build()
    }
}

pub fn builder<Context>(
    f: Context,
) -> WebViewApp<
    'static,
    String,
    impl Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    Context,
>
where
    Context: SpawnerFactory,
{
    WebViewApp {
        webview_builder: web_view::builder(),
        container: WVPreContainer {
            api_container: HashMap::new(),
            middleware_container: none(),
        },
        context: f,
        data_container: HashMap::new(),
    }
}
