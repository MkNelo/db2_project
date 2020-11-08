use std::sync::{Arc};

use super::super::middleware::Middleware;
use super::InvokeRequest;
use super::SpawnerFactory;
use super::{Api, ApiResponse, WebViewApp};
use super::{Message, Response};
use crate::Load;
use futures::future::ready;
use futures::{future::BoxFuture, Future, FutureExt};
use serde::Serialize;
use tokio::sync::RwLock;

pub struct WebViewApi<'a, API: 'a> {
    pub(crate) api_key: &'a str,
    pub(crate) api: API,
}

impl<'a, API> Api for (&'a str, API)
where
API: Api,
API::Response: Future + Send + 'a,
<API::Response as Future>::Output: Serialize + Send + 'a,
API::Message: Message,
{
    type Message = API::Message;
    type Response = API::Response;
    
    fn handle(&self, msg: Self::Message) -> Self::Response {
        let (_, ref api) = self;
        api.handle(msg)
    }
}

impl<'a, API> Api for WebViewApi<'a, API>
where
API: Api,
API::Response: Future + Send + 'a,
<API::Response as Future>::Output: Serialize + Send + 'a,
API::Message: Message,
{
    type Message = InvokeRequest;
    type Response = BoxFuture<'a, ApiResponse>;
    
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

impl<'a, API> From<(&'a str, API)> for WebViewApi<'a, (&'a str, API)>
where
API: Api,
API::Response: Future + Send + 'a,
<API::Response as Future>::Output: Serialize + Send + 'a,
API::Message: Message,
{
    fn from(apituple: (&'a str, API)) -> Self {
        let api_key = apituple.0;
        WebViewApi {
            api: apituple,
            api_key,
        }
    }
}

pub struct WVLazyApi<'a, Factory, API>(Arc<Factory>, Arc<RwLock<Option<API>>>, &'a str);

impl<'a, Factory, API> WVLazyApi<'a, Factory, API> {
    pub fn new(f: Factory, key: &'a str) -> Self {
        WVLazyApi(Arc::new(f), Arc::new(RwLock::new(None)), key)
    }
}

impl<Factory, Fut, API> Api for WVLazyApi<'static, Factory, API>
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
            if { ptr.read().await.is_none() } {
                let mut cell = ptr.write().await;
                let operation = if cell.is_none() {
                    factory(&mut msg).map(Some).right_future()
                } else {
                    ready(None).left_future()
                }
                .await;
                if let Some(api) = operation {
                    cell.replace(api);
                };
            };
            let cell = ptr.read().await;
            message
            .map(|msg| {
                cell
                .as_ref()
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

impl<Cont, API, Mid, Context> Load<WebViewApi<'static, API>>
for WebViewApp<'static, Cont, Mid, Context>
where
Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>> + Send,
API: Api + 'static,
API::Response: Future + Send + 'static,
<API::Response as Future>::Output: Serialize + Send + 'static,
API::Message: Message,
Context: SpawnerFactory,
Cont: AsRef<str>,
{
    type Result = Self;
    fn load(mut self, wapi: WebViewApi<'static, API>) -> Self::Result {
        self.container.insert(wapi.api_key, Box::new(wapi));
        self
    }
}

impl<Cont, Factory, Fut, API, Mid, Context> Load<WVLazyApi<'static, Factory, API>>
for WebViewApp<'static, Cont, Mid, Context>
where
Mid: Middleware<Message = InvokeRequest, Response = BoxFuture<'static, ApiResponse>> + Send,
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
    fn load(mut self, wapi: WVLazyApi<'static, Factory, API>) -> Self::Result {
        self.container.insert(wapi.2, Box::new(wapi));
        self
    }
}
