use std::sync::Arc;

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use self::api::WebViewLazyApi;
use self::container::ActorContainer;
use self::prelude::WebViewLoadableActor;

mod api;
pub mod app;
pub(crate) mod container;
pub mod middleware;
mod tests;

#[derive(Serialize, Deserialize, Clone)]
pub struct InvokeBody {
    pub(crate) api_name: String,
    pub(crate) payload: Value,
}

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct InvokeRequest {
    pub(crate) body: InvokeBody,
    pub(crate) caller: Recipient<AppString>,
    pub(crate) data: Arc<ActorContainer>,
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

impl InvokeRequest {
    pub fn body(&self) -> &InvokeBody {
        &self.body
    }

    pub fn container(self) -> Arc<ActorContainer> {
        self.data
    }
}

impl InvokeBody {
    pub fn api_name(&self) -> &String {
        &self.api_name
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }
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

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) struct AppString(String);

impl From<String> for AppString {
    fn from(param: String) -> Self {
        AppString(param)
    }
}

pub fn request<Data: Serialize>(api_name: String, data: &Data) -> InvokeBody {
    InvokeBody {
        api_name,
        payload: serde_json::to_value(data).unwrap(),
    }
}

pub fn error<Data: Serialize>(api_name: String, data: &Data) -> ApiResponse {
    ApiResponse::OpResponse {
        api_name: api_name.into(),
        body: ApiResponseBody {
            error: true,
            payload: serde_json::to_value(data).ok(),
        },
    }
}

pub fn void(api_name: String) -> ApiResponse {
    ApiResponse::OpDoNothing(api_name)
}

pub fn success<Data: Serialize>(api_name: String, data: &Data) -> ApiResponse {
    ApiResponse::OpResponse {
        api_name: api_name.into(),
        body: ApiResponseBody {
            error: false,
            payload: serde_json::to_value(data).ok(),
        },
    }
}

pub fn lazy<F>(api_key: &'static str, factory: F) -> WebViewLazyApi<F>
where
    WebViewLazyApi<F>: KeyedActor,
{
    WebViewLazyApi {
        api_key,
        factory,
        api: None,
    }
}

pub fn actor<A: Actor>(addr: Addr<A>) -> WebViewLoadableActor<A> {
    WebViewLoadableActor(addr)
}

pub trait KeyedActor: Actor<Context = Context<Self>> + Handler<InvokeRequest> {
    fn api_key(&self) -> &'static str;
}

pub mod prelude {
    pub use super::api::*;
    pub use super::app::*;
    pub use super::*;
}
