use std::future::ready;

use actix::dev::MessageResponse;
use actix::prelude::*;
use futures::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;

use crate::Api;

use super::error;
use super::success;
use super::InvokeBody;
use super::InvokeRequest;
use super::KeyedActor;

pub enum LazyResponse<A> {
    InitResponse(AtomicResponse<A, ()>),
    NormalResponse(ResponseActFuture<A, ()>),
}

impl<A> LazyResponse<A>
where
    A: Actor,
{
    pub fn init<F: ActorFuture<Actor = A, Output = ()> + 'static>(f: F) -> Self {
        Self::InitResponse(AtomicResponse::new(Box::pin(f)))
    }

    pub fn normal<F: ActorFuture<Actor = A, Output = ()> + 'static>(f: F) -> Self {
        Self::NormalResponse(Box::pin(f))
    }
}

impl<A, M> MessageResponse<A, M> for LazyResponse<A>
where
    A: Actor,
    M: Message<Result = ()>,
    A::Context: AsyncContext<A>,
{
    fn handle<R: dev::ResponseChannel<M>>(self, ctx: &mut A::Context, tx: Option<R>) {
        match self {
            LazyResponse::InitResponse(atomic) => atomic.handle(ctx, tx),
            LazyResponse::NormalResponse(future) => future.handle(ctx, tx),
        }
    }
}

pub struct WebViewApi<API> {
    pub(crate) api_key: &'static str,
    pub(crate) api: API,
}

impl<API> Actor for WebViewApi<API>
where
    API: Unpin + 'static,
{
    type Context = Context<Self>;
}

impl<API> Handler<InvokeRequest> for WebViewApi<API>
where
    API: Api + Unpin + 'static,
    API::Input: DeserializeOwned,
    API::Output: Future + Send,
    <API::Output as Future>::Output: Serialize,
{
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: InvokeRequest, _: &mut Self::Context) -> Self::Result {
        use serde_json::Value::*;
        let InvokeRequest {
            body: InvokeBody { api_name, payload },
            caller,
            data: _,
        } = msg;
        let result = serde_json::from_value(payload.clone());
        let handler = match result {
            Ok(result) => {
                self.api.handle(result).then(move |ref result| {
                    let value = serde_json::to_value(result).unwrap();
                    let send = match value {
                        Object(map) if map.contains_key("Err") => serde_json::to_string(&error(api_name, result)).unwrap(),
                        _ => serde_json::to_string(&success(api_name, result)).unwrap(),
                    };
                    caller.do_send(send.into()).ok();
                    ready(())
                })
                .right_future()
            },
            Err(e) => {
                let error = serde_json::to_string(&error(api_name, &json!({
                    "error": "Malformed payload",
                    "found": payload.to_string(),
                    "description": e.to_string()
                }))).unwrap();
                caller.do_send(error.into()).ok();
                ready(())
                    .left_future()
            }
        };
        Box::pin(handler.into_actor(self))
    }
}

pub struct WebViewLazyApi<F> {
    pub(crate) api_key: &'static str,
    pub(crate) api: Option<Recipient<InvokeRequest>>,
    pub(crate) factory: F,
}

impl<F> Actor for WebViewLazyApi<F>
where
    F: Unpin + 'static,
{
    type Context = Context<Self>;
}

impl<F, API, Fut> Handler<InvokeRequest> for WebViewLazyApi<F>
where
    API: Send,
    Fut: Future<Output = API> + 'static + Send,
    F: Fn(InvokeRequest) -> Fut + Unpin + 'static,
    WebViewApi<API>: Handler<InvokeRequest>,
    WebViewApi<API>: Actor<Context = Context<WebViewApi<API>>>,
{
    type Result = LazyResponse<Self>;

    fn handle(&mut self, msg: InvokeRequest, _: &mut Self::Context) -> Self::Result {
        if self.api.is_none() {
            let ref factory = self.factory;
            let api_key = self.api_key;
            let future = factory(msg.clone())
                .map(move |api| WebViewApi { api_key, api }.start().recipient());
            LazyResponse::init(future.into_actor(self).then(move |res, actor, _| {
                res.do_send(msg).ok();
                actor.api = Some(res);
                async {}.actfuture()
            }))
        } else {
            let api = self.api.as_ref().expect("Loaded true but cell empty");
            LazyResponse::normal(
                api.send(msg)
                    .into_actor(self)
                    .then(|_, _, _| async {}.actfuture()),
            )
        }
    }
}

impl<API> KeyedActor for WebViewApi<API>
where
    WebViewApi<API>: Actor<Context = Context<WebViewApi<API>>>,
    WebViewApi<API>: Handler<InvokeRequest>,
{
    fn api_key(&self) -> &'static str {
        self.api_key
    }
}

impl<F> KeyedActor for WebViewLazyApi<F>
where
    WebViewLazyApi<F>: Actor<Context = Context<WebViewLazyApi<F>>>,
    WebViewLazyApi<F>: Handler<InvokeRequest>,
{
    fn api_key(&self) -> &'static str {
        self.api_key
    }
}

pub struct WebViewLoadableActor<A: Actor>(pub(crate) Addr<A>);
