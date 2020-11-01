use std::sync::Arc;

use super::super::middleware::Middleware;
use super::InvokeRequest;
use super::SpawnerFactory;
use super::{Api, ApiResponse, WebViewApp};
use super::{Message, Response};
use crate::Load;
use futures::future::ready;
use futures::lock::Mutex;
use futures::{future::BoxFuture, Future, FutureExt};
use serde::Serialize;

pub struct WebViewApi<API> {
    pub(crate) api_key: &'static str,
    pub(crate) api: API,
}

impl<API> Api for (&'static str, API)
where
    API: Api,
    API::Response: Future + Send + 'static,
    <API::Response as Future>::Output: Serialize + Send + 'static,
    API::Message: Message,
{
    type Message = API::Message;
    type Response = API::Response;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        let (_, ref api) = self;
        api.handle(msg)
    }
}

impl<API> Api for WebViewApi<API>
where
    API: Api,
    API::Response: Future + Send + 'static,
    <API::Response as Future>::Output: Serialize + Send + 'static,
    API::Message: Message,
{
    type Message = InvokeRequest;
    type Response = BoxFuture<'static, ApiResponse>;

    fn handle(&self, msg: Self::Message) -> Self::Response {
        let api_name = msg.body().api_name.clone();
        let api_f_name = api_name.clone();
        let ref api = self.api;
        Message::from_message(&msg)
            .map(|body| api.handle(body))
            .map(|response| response.into_response())
            .map(move |future| {
                future
                    .then(move |val| {
                        futures::future::ready({
                            val.map(|response| ApiResponse::OpResponse {
                                api_name: api_name.clone(),
                                body: response,
                            })
                            .unwrap_or_else(|| ApiResponse::OpDoNothing(api_name))
                        })
                    })
                    .boxed()
            })
            .unwrap_or_else(|| {
                futures::future::ready(ApiResponse::OpDoNothing(api_f_name.clone())).boxed()
            })
    }
}

impl<API> From<(&'static str, API)> for WebViewApi<(&'static str, API)>
where
    API: Api,
    API::Response: Future + Send + 'static,
    <API::Response as Future>::Output: Serialize + Send + 'static,
    API::Message: Message,
{
    fn from(apituple: (&'static str, API)) -> Self {
        let api_key = apituple.0;
        WebViewApi {
            api: apituple,
            api_key,
        }
    }
}

pub struct WVLazyApi<Factory, API>(Arc<Factory>, Arc<Mutex<Option<API>>>, &'static str);

impl<Factory, API> WVLazyApi<Factory, API> {
    pub fn new(f: Factory, key: &'static str) -> Self {
        WVLazyApi(Arc::new(f), Arc::new(Mutex::new(None)), key)
    }
}

impl<Factory, Fut, API> Api for WVLazyApi<Factory, API>
where
    Factory: Fn(&mut InvokeRequest) -> Fut + Sync + Send + 'static,
    Fut: Future<Output = API> + Send + 'static,
    API: Api + Send + 'static + Sync,
    API::Response: Future + Send + 'static,
    <API::Response as Future>::Output: Serialize + Send + 'static,
    API::Message: Message + Send,
{
    type Message = InvokeRequest;

    type Response = BoxFuture<'static, ApiResponse>;

    fn handle(&self, mut msg: Self::Message) -> Self::Response {
        let message = Message::from_message(&msg);
        let api_name = msg.body().api_name().to_owned();
        let api_f_name = api_name.clone();
        let factory = self.0.clone();
        let ptr = Arc::clone(&self.1);
        async move {
            let mut cell = ptr.lock().await;
            let operation = if cell.is_none() {
                factory(&mut msg).map(Some).right_future()
            } else {
                ready(None).left_future()
            }
            .await;
            if let Some(api) = operation {
                cell.replace(api);
            }
            message
                .map(|msg| {
                    cell.as_ref()
                        .unwrap()
                        .handle(msg)
                        .into_response()
                        .then(|body| {
                            ready(match body {
                                Some(body) => ApiResponse::OpResponse {
                                    api_name: api_name.clone(),
                                    body,
                                },
                                None => ApiResponse::OpDoNothing(api_name),
                            })
                        })
                        .right_future()
                })
                .unwrap_or_else(|| ready(ApiResponse::OpDoNothing(api_f_name)).left_future())
                .await
        }
        .boxed()
    }
}

impl<'a, Cont, API, Mid, Context> Load<WebViewApi<API>> for WebViewApp<'a, Cont, Mid, Context>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    API: Api + 'static,
    API::Response: Future + Send + 'static,
    <API::Response as Future>::Output: Serialize + Send + 'static,
    API::Message: Message,
    Context: SpawnerFactory,
    Cont: AsRef<str>,
{
    type Result = Self;
    fn load(mut self, wapi: WebViewApi<API>) -> Self::Result {
        self.container.insert(wapi.api_key, Box::new(wapi));
        self
    }
}

impl<'a, Cont, Factory, Fut, API, Mid, Context> Load<WVLazyApi<Factory, API>>
    for WebViewApp<'a, Cont, Mid, Context>
where
    Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>>,
    Factory: Fn(&mut InvokeRequest) -> Fut + 'static + Sync + Send,
    Fut: Future<Output = API> + Send + 'static,
    API: Api + Send + 'static + std::marker::Sync,
    API::Response: Future + Send + 'static,
    <API::Response as Future>::Output: Serialize + Send + 'static,
    API::Message: Message + Send,
    Context: SpawnerFactory,
    Cont: AsRef<str>,
{
    type Result = Self;
    fn load(mut self, wapi: WVLazyApi<Factory, API>) -> Self::Result {
        self.container.insert(wapi.2, Box::new(wapi));
        self
    }
}
